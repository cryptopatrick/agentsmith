//! Integration tests for agentsmith

use agentsmith::{AgentHistory, Trace};
use rig::completion::Message;
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_create_history_in_memory() {
    let history = AgentHistory::new(":memory:", Some("test-session"))
        .await
        .expect("Failed to create history");

    assert_eq!(history.session_id(), "test-session");
}

#[tokio::test]
async fn test_create_history_auto_session_id() {
    let history = AgentHistory::new(":memory:", None)
        .await
        .expect("Failed to create history");

    // Should have a generated UUID session ID
    assert!(!history.session_id().is_empty());
    assert_eq!(history.session_id().len(), 36); // UUID length
}

#[tokio::test]
async fn test_log_and_retrieve_messages() {
    let history = AgentHistory::new(":memory:", Some("test")).await.unwrap();

    // Log a user message
    let user_msg = Message {
        role: "user".to_string(),
        content: "Hello, agent!".to_string(),
    };

    let mut metadata = HashMap::new();
    metadata.insert("test_key".to_string(), json!("test_value"));

    let trace = history.log_turn(&user_msg, metadata).await.unwrap();

    assert_eq!(trace.role, "user");
    assert_eq!(trace.content, "Hello, agent!");
    assert_eq!(trace.session_id, "test");

    // Log an assistant message
    let assistant_msg = Message {
        role: "assistant".to_string(),
        content: "Hello, human!".to_string(),
    };

    history.log_turn(&assistant_msg, HashMap::new()).await.unwrap();

    // Retrieve recent messages
    let recent = history.recent(10).await.unwrap();
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].content, "Hello, agent!");
    assert_eq!(recent[1].content, "Hello, human!");
}

#[tokio::test]
async fn test_recent_messages_limit() {
    let history = AgentHistory::new(":memory:", Some("test")).await.unwrap();

    // Log 10 messages
    for i in 0..10 {
        let msg = Message {
            role: "user".to_string(),
            content: format!("Message {}", i),
        };
        history.log_turn(&msg, HashMap::new()).await.unwrap();
    }

    // Request only 5 most recent
    let recent = history.recent(5).await.unwrap();
    assert_eq!(recent.len(), 5);

    // Should be messages 5-9 in chronological order
    assert_eq!(recent[0].content, "Message 5");
    assert_eq!(recent[4].content, "Message 9");
}

#[tokio::test]
async fn test_fts_search() {
    let history = AgentHistory::new(":memory:", Some("test")).await.unwrap();

    // Populate with searchable content
    let messages = vec![
        "How do I parse JSON in Rust?",
        "Use serde_json crate for parsing",
        "What about XML parsing?",
        "Try quick-xml crate",
        "JSON is better than XML",
    ];

    for content in messages {
        let msg =
            Message { role: "user".to_string(), content: content.to_string() };
        history.log_turn(&msg, HashMap::new()).await.unwrap();
    }

    // Search for JSON-related messages
    let results = history.search("JSON", 10, false).await.unwrap();

    // Should find messages containing "JSON"
    assert!(results.len() >= 2);
    assert!(results.iter().any(|t| t.content.contains("JSON")));

    // Search for parsing
    let results = history.search("parsing", 10, false).await.unwrap();
    assert!(results.len() >= 2);
}

#[tokio::test]
async fn test_search_with_limit() {
    let history = AgentHistory::new(":memory:", Some("test")).await.unwrap();

    // Add many messages with common term
    for i in 0..10 {
        let msg = Message {
            role: "user".to_string(),
            content: format!("Testing message number {}", i),
        };
        history.log_turn(&msg, HashMap::new()).await.unwrap();
    }

    // Search with limit
    let results = history.search("Testing", 3, false).await.unwrap();
    assert!(results.len() <= 3);
}

#[tokio::test]
async fn test_trace_metadata() {
    let history = AgentHistory::new(":memory:", Some("test")).await.unwrap();

    let msg = Message {
        role: "assistant".to_string(),
        content: "Response".to_string(),
    };

    let mut metadata = HashMap::new();
    metadata.insert("duration_ms".to_string(), json!(150));
    metadata.insert("success".to_string(), json!(true));
    metadata.insert("tokens_used".to_string(), json!(42));

    let trace = history.log_turn(&msg, metadata).await.unwrap();

    assert_eq!(trace.get_metadata("duration_ms"), Some(&json!(150)));
    assert_eq!(trace.get_metadata("success"), Some(&json!(true)));
    assert!(trace.is_success());
}

#[tokio::test]
async fn test_trace_success_filtering() {
    let history = AgentHistory::new(":memory:", Some("test")).await.unwrap();

    // Add successful message
    let msg1 = Message {
        role: "user".to_string(),
        content: "successful query".to_string(),
    };
    let mut meta1 = HashMap::new();
    meta1.insert("success".to_string(), json!(true));
    history.log_turn(&msg1, meta1).await.unwrap();

    // Add failed message
    let msg2 = Message {
        role: "user".to_string(),
        content: "failed query".to_string(),
    };
    let mut meta2 = HashMap::new();
    meta2.insert("success".to_string(), json!(false));
    history.log_turn(&msg2, meta2).await.unwrap();

    // Search for all
    let all_results = history.search("query", 10, false).await.unwrap();
    assert_eq!(all_results.len(), 2);

    // Search for successful only
    let success_results = history.search("query", 10, true).await.unwrap();
    assert_eq!(success_results.len(), 1);
    assert!(success_results[0].content.contains("successful"));
}

#[tokio::test]
async fn test_import_jsonl() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let history = AgentHistory::new(":memory:", Some("test")).await.unwrap();

    // Create a temporary JSONL file
    let mut temp_file = NamedTempFile::new().unwrap();

    let trace1 = Trace::new(
        "test".to_string(),
        "user".to_string(),
        "First message".to_string(),
    );
    let trace2 = Trace::new(
        "test".to_string(),
        "assistant".to_string(),
        "Second message".to_string(),
    );

    writeln!(temp_file, "{}", serde_json::to_string(&trace1).unwrap())
        .unwrap();
    writeln!(temp_file, "{}", serde_json::to_string(&trace2).unwrap())
        .unwrap();

    let path = temp_file.path().to_string_lossy().to_string();

    // Import
    let count = history.import_jsonl(&path).await.unwrap();
    assert_eq!(count, 2);

    // Verify imported data
    let recent = history.recent(10).await.unwrap();
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].content, "First message");
    assert_eq!(recent[1].content, "Second message");
}

#[tokio::test]
async fn test_multiple_sessions() {
    let history1 =
        AgentHistory::new(":memory:", Some("session-1")).await.unwrap();
    let history2 =
        AgentHistory::new(":memory:", Some("session-2")).await.unwrap();

    let msg =
        Message { role: "user".to_string(), content: "Test".to_string() };

    history1.log_turn(&msg, HashMap::new()).await.unwrap();

    // Each session should be independent (in memory mode, they have separate DBs)
    let recent1 = history1.recent(10).await.unwrap();
    let recent2 = history2.recent(10).await.unwrap();

    assert_eq!(recent1.len(), 1);
    assert_eq!(recent2.len(), 0);
}

#[tokio::test]
async fn test_trace_serialization() {
    let trace = Trace::new(
        "session-123".to_string(),
        "user".to_string(),
        "Hello".to_string(),
    );

    // Serialize to JSON
    let json = serde_json::to_string(&trace).unwrap();
    assert!(json.contains("session-123"));
    assert!(json.contains("Hello"));

    // Deserialize back
    let deserialized: Trace = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.session_id, "session-123");
    assert_eq!(deserialized.content, "Hello");
}

#[tokio::test]
async fn test_empty_search() {
    let history = AgentHistory::new(":memory:", Some("test")).await.unwrap();

    // Search in empty history
    let results = history.search("anything", 10, false).await.unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_recent_messages_conversion() {
    let history = AgentHistory::new(":memory:", Some("test")).await.unwrap();

    let msg1 =
        Message { role: "user".to_string(), content: "Question".to_string() };
    let msg2 = Message {
        role: "assistant".to_string(),
        content: "Answer".to_string(),
    };

    history.log_turn(&msg1, HashMap::new()).await.unwrap();
    history.log_turn(&msg2, HashMap::new()).await.unwrap();

    // Get as Messages
    let messages = history.recent_messages(10).await.unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[0].content, "Question");
    assert_eq!(messages[1].role, "assistant");
    assert_eq!(messages[1].content, "Answer");
}
