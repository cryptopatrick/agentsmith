//! Trace data structures for storing agent interactions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A single trace entry representing one agent turn (message + metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    /// Unique identifier for this trace
    pub id: String,

    /// Session this trace belongs to
    pub session_id: String,

    /// Role (user, assistant, system)
    pub role: String,

    /// Message content
    pub content: String,

    /// Structured metadata
    #[serde(default)]
    pub metadata: HashMap<String, Value>,

    /// When this trace was created
    pub created_at: DateTime<Utc>,

    /// Optional embedding for semantic search (future use)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<String>,
}

impl Trace {
    /// Create a new trace
    pub fn new(session_id: String, role: String, content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            session_id,
            role,
            content,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            embedding: None,
        }
    }

    /// Add metadata to this trace
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add a single metadata field
    pub fn add_metadata(&mut self, key: String, value: Value) {
        self.metadata.insert(key, value);
    }

    /// Get metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&Value> {
        self.metadata.get(key)
    }

    /// Check if this trace was successful (based on metadata)
    pub fn is_success(&self) -> bool {
        self.metadata.get("success").and_then(|v| v.as_bool()).unwrap_or(true)
    }
}
