//! Persistent chat example - a REPL that remembers everything across restarts
//!
//! Run with: cargo run --example persistent_chat
//!
//! This demonstrates the core value proposition: close the program, restart it,
//! and the agent still remembers your previous conversations.

use agentsmith::{AgentHistory, SmartAgent};
use rig::providers::openai;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for OpenAI API key
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        eprintln!("Warning: OPENAI_API_KEY not set. Using mock mode.");
        "mock-key".to_string()
    });

    println!("🤖 AgentSmith Persistent Chat Demo");
    println!("===================================\n");
    println!("This agent remembers everything across restarts!");
    println!("Try closing and reopening - your conversation persists.\n");
    println!("Commands:");
    println!("  /history - Show recent conversation history");
    println!("  /search <query> - Search past conversations");
    println!("  /summary - Generate session summary");
    println!("  /quit - Exit\n");

    // Create persistent history (stored in ./chat_history.db)
    let history = AgentHistory::new("./chat_history.db", None).await?;
    println!("📁 Session ID: {}\n", history.session_id());

    // Create OpenAI client and agent
    let client = openai::Client::new(&api_key);
    let agent = client
        .agent("gpt-4")
        .preamble("You are a helpful AI assistant with a perfect memory. You can recall past conversations and provide contextual responses.")
        .build();

    // Wrap with SmartAgent for automatic memory
    let mut smart_agent = SmartAgent::new(agent, history.clone());

    // Show recent history if any
    let recent = history.recent(5).await?;
    if !recent.is_empty() {
        println!("📜 Recent history found ({} messages)", recent.len());
        for trace in recent {
            println!(
                "  [{}] {}: {}",
                trace.created_at.format("%H:%M:%S"),
                trace.role,
                trace.content.chars().take(60).collect::<String>()
            );
        }
        println!();
    }

    // REPL
    let stdin = io::stdin();
    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Handle commands
        match input {
            "/quit" | "/exit" => {
                println!("👋 Goodbye! Your conversation is saved.");
                break;
            }
            "/history" => {
                let traces = history.recent(10).await?;
                println!("\n📜 Recent History:");
                for trace in traces {
                    println!(
                        "  [{}] {}: {}",
                        trace.created_at.format("%Y-%m-%d %H:%M:%S"),
                        trace.role,
                        trace.content
                    );
                }
                println!();
                continue;
            }
            cmd if cmd.starts_with("/search ") => {
                let query = cmd.strip_prefix("/search ").unwrap();
                let results = history.search(query, 5, false).await?;
                println!("\n🔍 Search Results for '{}':", query);
                if results.is_empty() {
                    println!("  No matches found.");
                } else {
                    for trace in results {
                        println!(
                            "  [{}] {}: {}",
                            trace.created_at.format("%Y-%m-%d %H:%M:%S"),
                            trace.role,
                            trace.content
                        );
                    }
                }
                println!();
                continue;
            }
            "/summary" => {
                println!("Generating session summary...");
                match smart_agent.summarize().await {
                    Ok(summary) => println!("\n📝 Summary:\n{}\n", summary),
                    Err(e) => println!("❌ Error generating summary: {}", e),
                }
                continue;
            }
            _ => {}
        }

        // Chat with the smart agent
        match smart_agent.chat(input).await {
            Ok(response) => {
                println!("Agent: {}\n", response);
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
            }
        }
    }

    Ok(())
}
