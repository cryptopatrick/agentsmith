//! # AgentSmith
//!
//! AI Agent forging utilities - giving agents persistent, searchable memory.
//!
//! Inspired by Atuin for shell history, `agentsmith` provides a permanent, fuzzy-searchable,
//! metadata-rich memory layer for Rig agents that survives process restarts.
//!
//! ## Example
//!
//! ```rust,no_run
//! use agentsmith::{AgentHistory, SmartAgent};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create persistent history
//! let history = AgentHistory::new("my_agent.db", Some("session-123")).await?;
//!
//! // Wrap your Rig agent
//! // let agent = ...; // your rig agent
//! // let mut smart_agent = SmartAgent::new(agent, history);
//!
//! // Chat with automatic recall of relevant past interactions
//! // let reply = smart_agent.chat("How did we fix the JSON parsing bug?").await?;
//! # Ok(())
//! # }
//! ```

mod error;
mod history;
mod smart_agent;
mod trace;

pub use error::{Error, Result};
pub use history::AgentHistory;
pub use smart_agent::SmartAgent;
pub use trace::Trace;
