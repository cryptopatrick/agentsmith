//! SmartAgent wrapper that adds automatic history recall and summarization

use crate::{AgentHistory, Result};
use rig::{
    agent::Agent,
    completion::{Chat, CompletionModel, Message},
};
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;

/// A smart agent wrapper that automatically manages persistent memory
pub struct SmartAgent<M: CompletionModel> {
    agent: Agent<M>,
    history: AgentHistory,
    recall_top_k: usize,
    summarize_every: usize,
    turn_count: usize,
}

impl<M: CompletionModel + 'static> SmartAgent<M> {
    /// Create a new SmartAgent
    ///
    /// # Arguments
    /// * `agent` - The base Rig agent to wrap
    /// * `history` - AgentHistory instance for persistence
    ///
    /// # Example
    /// ```rust,no_run
    /// use agentsmith::{AgentHistory, SmartAgent};
    /// use rig::providers::openai;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create OpenAI agent
    /// let client = openai::Client::new("your-api-key");
    /// let agent = client.agent("gpt-4").build();
    ///
    /// // Create persistent history
    /// let history = AgentHistory::new("agent.db", Some("session-1")).await?;
    ///
    /// // Wrap with SmartAgent
    /// let smart_agent = SmartAgent::new(agent, history);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(agent: Agent<M>, history: AgentHistory) -> Self {
        Self {
            agent,
            history,
            recall_top_k: 4,
            summarize_every: 20,
            turn_count: 0,
        }
    }

    /// Set the number of relevant past traces to recall (default: 4)
    pub fn with_recall_top_k(mut self, k: usize) -> Self {
        self.recall_top_k = k;
        self
    }

    /// Set how often to auto-summarize the session (default: every 20 turns)
    pub fn with_summarize_every(mut self, n: usize) -> Self {
        self.summarize_every = n;
        self
    }

    /// Chat with the agent, automatically managing history and recall
    ///
    /// This method:
    /// 1. Searches history for relevant past traces
    /// 2. Injects them as context
    /// 3. Sends the user message
    /// 4. Logs the response with metadata
    /// 5. Periodically triggers summarization
    pub async fn chat(&mut self, user_input: &str) -> Result<String> {
        let start = Instant::now();

        // 1. Search for relevant past traces
        let relevant_traces =
            self.history.search(user_input, self.recall_top_k, false).await?;

        // 2. Build context with relevant past experiences
        let mut context_messages = Vec::new();

        if !relevant_traces.is_empty() {
            let mut recall_context =
                String::from("Relevant past experiences:\n\n");
            for (i, trace) in relevant_traces.iter().enumerate() {
                recall_context.push_str(&format!(
                    "{}. [{}] {}: {}\n",
                    i + 1,
                    trace.created_at.format("%Y-%m-%d %H:%M"),
                    trace.role,
                    trace.content
                ));
            }

            context_messages.push(Message {
                role: "system".to_string(),
                content: recall_context,
            });
        }

        // 3. Append current user message
        let user_message = Message {
            role: "user".to_string(),
            content: user_input.to_string(),
        };

        // Log user turn
        let mut user_metadata = HashMap::new();
        user_metadata.insert(
            "recalled_traces".to_string(),
            json!(relevant_traces.len()),
        );
        self.history.log_turn(&user_message, user_metadata).await?;

        // 4. Call the underlying agent
        let response = self
            .agent
            .chat(user_input, context_messages)
            .await
            .map_err(|e| crate::Error::Rig(e.to_string()))?;

        let duration = start.elapsed();

        // 5. Log assistant response with metadata
        let assistant_message = Message {
            role: "assistant".to_string(),
            content: response.clone(),
        };

        let mut metadata = HashMap::new();
        metadata
            .insert("duration_ms".to_string(), json!(duration.as_millis()));
        metadata.insert("success".to_string(), json!(true));

        // Try to extract token usage if available (Rig doesn't expose this directly yet)
        // This is a placeholder for when Rig adds token usage tracking
        metadata.insert("tokens_used".to_string(), json!(null));

        self.history.log_turn(&assistant_message, metadata).await?;

        // 6. Increment turn count and check if we should summarize
        self.turn_count += 1;
        if self.turn_count.is_multiple_of(self.summarize_every) {
            // Trigger background summarization (we could make this async in the background)
            let _ = self.history.summarize_session(&self.agent).await;
        }

        Ok(response)
    }

    /// Get a reference to the underlying agent
    pub fn agent(&self) -> &Agent<M> {
        &self.agent
    }

    /// Get a mutable reference to the underlying agent
    pub fn agent_mut(&mut self) -> &mut Agent<M> {
        &mut self.agent
    }

    /// Get a reference to the history
    pub fn history(&self) -> &AgentHistory {
        &self.history
    }

    /// Get the current turn count
    pub fn turn_count(&self) -> usize {
        self.turn_count
    }

    /// Manually trigger session summarization
    pub async fn summarize(&self) -> Result<String> {
        self.history.summarize_session(&self.agent).await
    }
}
