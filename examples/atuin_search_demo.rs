//! Atuin-style search demo - shows fuzzy search and recall injection
//!
//! Run with: cargo run --example atuin_search_demo

use agentsmith::{AgentHistory, Trace};
use rig::completion::Message;
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 AgentSmith Atuin-Style Search Demo");
    println!("=====================================\n");

    // Create in-memory history for demo
    let history = AgentHistory::new(":memory:", Some("demo-session")).await?;

    // Populate with sample traces (simulating past agent interactions)
    println!("📝 Populating sample history...\n");

    let samples = vec![
        ("user", "How do I parse JSON in Rust?"),
        (
            "assistant",
            "Use serde_json crate: serde_json::from_str(&json_string)?",
        ),
        ("user", "I'm getting a parsing error with nested JSON"),
        (
            "assistant",
            "Make sure your struct derives Deserialize and matches the JSON structure",
        ),
        ("user", "How do I handle optional fields?"),
        ("assistant", "Use Option<T> in your struct: field: Option<String>"),
        ("user", "What's the best way to work with dates?"),
        ("assistant", "Use the chrono crate for date/time handling"),
        ("user", "How do I serialize data to JSON?"),
        ("assistant", "Use serde_json::to_string(&your_struct)?"),
        ("user", "Can you help with file I/O?"),
        ("assistant", "Use std::fs::read_to_string for reading files"),
    ];

    for (i, (role, content)) in samples.iter().enumerate() {
        let mut trace = Trace::new(
            "demo-session".to_string(),
            role.to_string(),
            content.to_string(),
        );

        // Add some metadata
        let mut metadata = HashMap::new();
        metadata.insert("duration_ms".to_string(), json!(100 + i * 10));
        metadata.insert("success".to_string(), json!(true));
        trace = trace.with_metadata(metadata);

        // Use log_turn to insert (we need to convert Trace to Message)
        let message =
            Message { role: role.to_string(), content: content.to_string() };
        history.log_turn(&message, trace.metadata).await?;
    }

    println!("✅ Inserted {} sample interactions\n", samples.len());

    // Demonstrate fuzzy search
    println!("🔍 Search Examples:\n");

    let queries = vec!["JSON parse", "error", "optional", "date time", "file"];

    for query in queries {
        println!("Query: \"{}\"", query);
        let results = history.search(query, 3, false).await?;

        if results.is_empty() {
            println!("  No results found\n");
        } else {
            println!("  Found {} results:", results.len());
            for (i, trace) in results.iter().enumerate() {
                println!(
                    "    {}. [{}] {}: {}",
                    i + 1,
                    trace.role,
                    trace.content.chars().take(50).collect::<String>(),
                    if trace.content.len() > 50 { "..." } else { "" }
                );
            }
            println!();
        }
    }

    // Demonstrate success filtering
    println!("🎯 Success-only search:");
    let all_results = history.search("JSON", 10, true).await?;
    println!("  Found {} successful traces about JSON\n", all_results.len());

    // Show recent traces
    println!("📜 Recent conversation history:");
    let recent = history.recent(5).await?;
    for trace in recent {
        println!(
            "  [{}] {}: {}",
            trace.created_at.format("%H:%M:%S"),
            trace.role,
            trace.content
        );
    }

    println!("\n✨ Demo complete! This shows how AgentSmith provides");
    println!("   Atuin-style fuzzy search over agent conversation history.");

    Ok(())
}
