//! Core AgentHistory implementation for persistent agent memory

use crate::{Error, Result, Trace};
use chrono::Utc;
use rig::{
    agent::Agent,
    completion::{Chat, CompletionModel, Message},
};
use serde_json::Value;
use sqlx::{Row, sqlite::SqlitePool};
use std::{collections::HashMap, path::Path};

/// Persistent history storage for agent interactions
#[derive(Clone)]
pub struct AgentHistory {
    pool: SqlitePool,
    session_id: String,
}

impl AgentHistory {
    /// Create a new AgentHistory instance
    ///
    /// # Arguments
    /// * `path` - Path to SQLite database file (":memory:" for in-memory)
    /// * `session_id` - Optional session identifier (generates UUID if None)
    ///
    /// # Example
    /// ```rust,no_run
    /// # use agentsmith::AgentHistory;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let history = AgentHistory::new("agent.db", Some("session-1")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(
        path: impl AsRef<Path>,
        session_id: Option<&str>,
    ) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let db_url = if path_str == ":memory:" {
            "sqlite::memory:".to_string()
        } else {
            format!("sqlite://{}", path_str)
        };

        let pool = SqlitePool::connect(&db_url).await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        let session_id = session_id
            .map(String::from)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        // Create session if it doesn't exist
        sqlx::query(
            "INSERT OR IGNORE INTO sessions (id, updated_at) VALUES (?, datetime('now'))",
        )
        .bind(&session_id)
        .execute(&pool)
        .await?;

        Ok(Self { pool, session_id })
    }

    /// Get the current session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Log a single agent turn (message) to the history
    ///
    /// # Arguments
    /// * `message` - The Rig message to log
    /// * `metadata` - Additional metadata (duration_ms, success, tokens_used, etc.)
    pub async fn log_turn(
        &self,
        message: &Message,
        metadata: HashMap<String, Value>,
    ) -> Result<Trace> {
        let trace = Trace::new(
            self.session_id.clone(),
            message.role.clone(),
            message.content.clone(),
        )
        .with_metadata(metadata);

        self.log_trace(&trace).await?;

        // Update session timestamp
        sqlx::query(
            "UPDATE sessions SET updated_at = datetime('now') WHERE id = ?",
        )
        .bind(&self.session_id)
        .execute(&self.pool)
        .await?;

        Ok(trace)
    }

    /// Log a trace directly
    async fn log_trace(&self, trace: &Trace) -> Result<()> {
        let metadata_json = serde_json::to_string(&trace.metadata)?;
        let created_at = trace.created_at.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO traces (id, session_id, role, content, metadata, created_at, embedding)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&trace.id)
        .bind(&trace.session_id)
        .bind(&trace.role)
        .bind(&trace.content)
        .bind(&metadata_json)
        .bind(&created_at)
        .bind(&trace.embedding)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Search traces using FTS5 fuzzy search (Atuin-style)
    ///
    /// # Arguments
    /// * `query` - Search query string
    /// * `limit` - Maximum number of results
    /// * `success_only` - Only return traces marked as successful
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        success_only: bool,
    ) -> Result<Vec<Trace>> {
        // Build FTS5 query - use MATCH for full-text search
        let fts_query = if query.is_empty() {
            // If empty query, return recent traces
            return self.recent(limit).await;
        } else {
            query.to_string()
        };

        let sql = r#"
            SELECT t.id, t.session_id, t.role, t.content, t.metadata, t.created_at, t.embedding
            FROM traces t
            JOIN traces_fts fts ON t.rowid = fts.rowid
            WHERE traces_fts MATCH ?
            ORDER BY rank, t.created_at DESC
            LIMIT ?
            "#;

        let rows = sqlx::query(sql)
            .bind(&fts_query)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?;

        let mut traces = Vec::new();
        for row in rows {
            let trace = self.row_to_trace(row)?;
            if !success_only || trace.is_success() {
                traces.push(trace);
            }
        }

        Ok(traces)
    }

    /// Get the N most recent traces
    pub async fn recent(&self, n: usize) -> Result<Vec<Trace>> {
        let rows = sqlx::query(
            r#"
            SELECT id, session_id, role, content, metadata, created_at, embedding
            FROM traces
            WHERE session_id = ?
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(&self.session_id)
        .bind(n as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut traces = Vec::new();
        for row in rows {
            traces.push(self.row_to_trace(row)?);
        }

        // Reverse to get chronological order
        traces.reverse();
        Ok(traces)
    }

    /// Get recent traces as Rig Messages for context injection
    pub async fn recent_messages(&self, n: usize) -> Result<Vec<Message>> {
        let traces = self.recent(n).await?;
        Ok(traces.into_iter().map(trace_to_message).collect())
    }

    /// Generate a summary of the current session using an agent
    pub async fn summarize_session<M: CompletionModel>(
        &self,
        summarizer: &Agent<M>,
    ) -> Result<String> {
        // Get all traces from this session
        let rows = sqlx::query(
            r#"
            SELECT id, session_id, role, content, metadata, created_at, embedding
            FROM traces
            WHERE session_id = ?
            ORDER BY created_at ASC
            "#,
        )
        .bind(&self.session_id)
        .fetch_all(&self.pool)
        .await?;

        let mut traces = Vec::new();
        for row in rows {
            traces.push(self.row_to_trace(row)?);
        }

        // Build conversation history for summarization
        let mut conversation = String::new();
        for trace in &traces {
            conversation
                .push_str(&format!("{}: {}\n", trace.role, trace.content));
        }

        // Ask the agent to summarize
        let summary_prompt = format!(
            "Please provide a concise summary of the following conversation:\n\n{}",
            conversation
        );

        let summary = summarizer
            .chat(&summary_prompt, vec![])
            .await
            .map_err(|e| Error::Rig(e.to_string()))?;

        // Store summary in sessions table
        sqlx::query("UPDATE sessions SET summary = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(&summary)
            .bind(&self.session_id)
            .execute(&self.pool)
            .await?;

        Ok(summary)
    }

    /// Import traces from a JSONL file (for migrating old logs)
    pub async fn import_jsonl(&self, path: &str) -> Result<usize> {
        let content = tokio::fs::read_to_string(path).await?;
        let mut count = 0;

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let trace: Trace = serde_json::from_str(line)?;
            self.log_trace(&trace).await?;
            count += 1;
        }

        Ok(count)
    }

    /// Convert a SQLx row to a Trace
    fn row_to_trace(&self, row: sqlx::sqlite::SqliteRow) -> Result<Trace> {
        let metadata_str: String = row.try_get("metadata")?;
        let metadata: HashMap<String, Value> =
            serde_json::from_str(&metadata_str)?;

        let created_at_str: String = row.try_get("created_at")?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| Error::Other(format!("Invalid datetime: {}", e)))?
            .with_timezone(&Utc);

        Ok(Trace {
            id: row.try_get("id")?,
            session_id: row.try_get("session_id")?,
            role: row.try_get("role")?,
            content: row.try_get("content")?,
            metadata,
            created_at,
            embedding: row.try_get("embedding")?,
        })
    }
}

/// Convert a Trace to a Rig Message
fn trace_to_message(trace: Trace) -> Message {
    Message { role: trace.role, content: trace.content }
}
