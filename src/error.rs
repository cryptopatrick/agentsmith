//! Error types for agentsmith

use thiserror::Error;

/// Result type alias for agentsmith operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in agentsmith operations
#[derive(Error, Debug)]
pub enum Error {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    /// Rig error
    #[error("Rig error: {0}")]
    Rig(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}
