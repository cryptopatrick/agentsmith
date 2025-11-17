//! Import old logs from JSONL format
//!
//! Run with: cargo run --example import_old_logs <path-to-jsonl>

use agentsmith::AgentHistory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 AgentSmith Log Import Tool");
    println!("=============================\n");

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <jsonl-file>", args[0]);
        eprintln!(
            "\nThe JSONL file should contain one Trace object per line."
        );
        eprintln!("Example trace format:");
        eprintln!(
            r#"{{"id":"...","session_id":"...","role":"user","content":"...","metadata":{{}},"created_at":"2025-11-17T..."}}"#
        );
        std::process::exit(1);
    }

    let jsonl_path = &args[1];
    println!("📁 Importing from: {}", jsonl_path);

    // Create history (or connect to existing)
    let history = AgentHistory::new("./imported_history.db", None).await?;
    println!("💾 Database: ./imported_history.db");
    println!("🆔 Session ID: {}\n", history.session_id());

    // Import
    println!("⏳ Importing...");
    match history.import_jsonl(jsonl_path).await {
        Ok(count) => {
            println!("✅ Successfully imported {} traces!", count);

            // Show some stats
            let recent = history.recent(5).await?;
            println!("\n📊 Sample of imported data:");
            for trace in recent {
                println!(
                    "  [{}] {}: {}",
                    trace.created_at.format("%Y-%m-%d %H:%M"),
                    trace.role,
                    trace.content.chars().take(60).collect::<String>()
                );
            }
        }
        Err(e) => {
            eprintln!("❌ Import failed: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n✨ Import complete!");
    println!(
        "You can now use this history with any AgentSmith-powered agent."
    );

    Ok(())
}
