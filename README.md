<h1 align="center">
  <br>
    <img 
      src="https://github.com/cryptopatrick/factory/blob/master/img/100days/agentsmith.png" 
      alt="AgentSmith" 
      width="200"
    />
  <br>
  AGENTSMITH
  <br>
</h1>

<h4 align="center">
  A Rust library for giving AI agents persistent, searchable memory that survives restarts.
</h4>

<p align="center">
  <a href="https://crates.io/crates/agentsmith" target="_blank">
    <img src="https://img.shields.io/crates/v/agentsmith" alt="Crates.io"/>
  </a>
  <a href="https://crates.io/crates/agentsmith" target="_blank">
    <img src="https://img.shields.io/crates/d/agentsmith" alt="Downloads"/>
  </a>
  <a href="https://docs.rs/agentsmith" target="_blank">
    <img src="https://docs.rs/agentsmith/badge.svg" alt="Documentation"/>
  </a>
  <a href="LICENSE" target="_blank">
    <img src="https://img.shields.io/github/license/sulu/sulu.svg" alt="GitHub license"/>
  </a>
</p>

<b>Author's bio:</b> 👋😀 Hi, I'm CryptoPatrick! I'm currently enrolled as an
Undergraduate student in Mathematics, at Chalmers & the University of Gothenburg, Sweden. <br>
If you have any questions or need more info, then please <a href="https://discord.gg/T8EWmJZpCB">join my Discord Channel: AiMath</a>

---

<p align="center">
  <a href="#-what-is-agentsmith">What is AgentSmith</a> •
  <a href="#-features">Features</a> •
  <a href="#-how-to-use">How To Use</a> •
  <a href="#-documentation">Documentation</a> •
  <a href="#-license">License</a>
</p>

## 🛎 Important Notices
* **Framework Support**: Currently designed for Rig agents, but extensible to other frameworks
* **Storage**: Uses SQLite for persistent storage with full-text search capabilities
* **Inspired by Atuin**: Brings the power of Atuin's searchable shell history to AI agents

<!-- TABLE OF CONTENTS -->
<h2 id="table-of-contents"> :pushpin: Table of Contents</h2>

<details open="open">
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#-what-is-agentsmith">What is AgentSmith</a></li>
    <li><a href="#-features">Features</a></li>
      <ul>
        <li><a href="#-core-functionality">Core Functionality</a></li>
        <li><a href="#-memory-capabilities">Memory Capabilities</a></li>
        <li><a href="#-smart-agent-features">Smart Agent Features</a></li>
        <li><a href="#-persistence">Persistence</a></li>
      </ul>
    <li><a href="#-architecture">Architecture</a></li>
    <li><a href="#-how-to-use">How to Use</a></li>
    <li><a href="#-examples">Examples</a></li>
    <li><a href="#-testing">Testing</a></li>
    <li><a href="#-documentation">Documentation</a></li>
    <li><a href="#-license">License</a>
  </ol>
</details>

## 🤔 What is AgentSmith

`agentsmith` is a Rust library that provides AI agents with persistent, fuzzy-searchable, metadata-rich memory that survives process restarts. Inspired by [Atuin](https://github.com/atuinsh/atuin) for shell history, AgentSmith ensures your agents never forget past conversations.

Built specifically for [Rig](https://github.com/0xPlaygrounds/rig) agents, AgentSmith wraps your existing agents with a memory layer that automatically:
- Records all conversations to a SQLite database
- Enables fuzzy search across historical interactions
- Provides session management for organizing conversations
- Allows agents to recall relevant context from past discussions

### Use Cases

- **Persistent AI Assistants**: Build chatbots that remember users across sessions
- **Context-Aware Agents**: Enable agents to reference past conversations naturally
- **Conversation Analytics**: Search and analyze agent interactions over time
- **Session Management**: Organize conversations by session for different contexts
- **Agent Memory Research**: Experiment with long-term memory for AI agents

## 📷 Features

`agentsmith` provides a complete memory layer for AI agents with persistent storage and intelligent retrieval:

### 🔧 Core Functionality
- **Persistent History**: Automatic storage of all agent interactions in SQLite
- **Fuzzy Search**: Full-text search across conversation history with relevance ranking
- **Session Management**: Group conversations into sessions for different contexts
- **Metadata Tracking**: Store timestamps, roles, and custom metadata with each interaction

### 🧠 Memory Capabilities
- **Automatic Recording**: All agent interactions are automatically saved
- **Smart Retrieval**: Retrieve recent messages or search by content
- **Context Injection**: Automatically include relevant past context in conversations
- **Cross-Session Search**: Search across all conversations or within specific sessions

### 🤖 Smart Agent Features
- **Transparent Wrapping**: Drop-in wrapper for existing Rig agents
- **Automatic Context**: Agents automatically recall relevant past interactions
- **Session Summaries**: Generate summaries of conversation sessions
- **Conversation History**: Access chronological history of interactions

### 💾 Persistence
- **SQLite Storage**: Reliable, file-based persistence with no external dependencies
- **Migration Support**: Built-in database schema migrations
- **Session Continuity**: Resume conversations across process restarts
- **Export Capabilities**: Import/export conversation history

## 📐 Architecture

1. 🏛 Overall Architecture
```diagram
┌──────────────────────────────────────────────────────────┐
│           User Application (CLI/Backend/App)             │
│              Single call: smart_agent.chat()             │
└──────────────────────┬───────────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────────┐
│                   SmartAgent Component                   │
│  • Search history for relevant traces                    │
│  • Inject past context as system message                 │
│  • Call underlying Rig agent                             │
│  • Log user & assistant turns with metadata              │
│  • Auto-summarize every N turns                          │
│  • Track success, duration, token usage                  │
└──────────────┬──────────────────────────┬────────────────┘
               │                          │
       ┌───────▼────────┐        ┌────────▼─────────┐
       │    Rig Agent   │        │   AgentHistory   │
       │  • GPT-4/etc   │        │  • SQLite Pool   │
       │  • Custom LLM  │        │  • FTS5 Search   │
       │  • Tools       │        │  • JSONL Import  │
       └────────────────┘        │  • Summarization │
                                 └─────────┬────────┘
                                           │
                                   ┌────────▼ ────────┐
                                   │  SQLite Database │
                                   │  • sessions      │
                                   │  • traces        │
                                   │  • traces_fts    │
                                   └──────────────────┘
```

User → SmartAgent → History Search → Rig Agent → Log Response → SQLite



2. 🚃 Data Flow Diagram

```diagram
┌──────────────────────────────────────────────────────────┐
│                 chat("How did we fix X?")                │
└──────────────────────┬───────────────────────────────────┘
                       │
              ┌────────▼ ────────┐
              │    1. FTS5       │
              │     Search       │──────┐
              │     top_k=4      │      │
              └──────────────────┘      │
                       │                │
                       │  relevant      │
                       │  traces        │
                       ▼                │
              ┌────────────────────┐    │
              │   2. Build         │    │
              │   Context          │◄───┘
              │   Messages         │
              └─────────┬──────────┘
                        │
                        │ system: "Relevant past...\n"
                        │ + user: "How did we fix X?"
                        ▼
              ┌────────────────────┐
              │   3. Rig Agent     │
              │       .chat()      │
              └─────────┬──────────┘
                        │
                        │ response
                        ▼
              ┌────────────────────┐
              │   4. Log Turn      │
              │   • User msg       │
              │   • Assistant      │
              │   • Metadata       │
              │     - duration     │
              │     - success      │
              │     - tokens       │
              └─────────┬──────────┘
                        │
                        ▼
              ┌────────────────────┐
              │   5. Store in      │
              │   traces table     │
              │   + FTS5 index     │
              └────────────────────┘
```

3. 💾 Storage Layer Architecture

```diagram
┌────────────────────────────────────────────────────────── ┐
│                        AgentHistory                       │
│  ┌─────────────────────────────────────────────────────┐  │
│  │                     Public API                      │  │
│  │  • new(path, session_id)                            │  │
│  │  • log_turn(message, metadata) → Trace              │  │
│  │  • search(query, limit, success_only) → Vec<Trace>  │  │
│  │  • recent(n) → Vec<Trace>                           │  │
│  │  • summarize_session(agent) → String                │  │
│  │  • import_jsonl(path) → usize                       │  │
│  └─────────────────────────────────────────────────────┘  │
│                               │                           │
│  ┌────────────────────────────▼─────────────────────────┐ │
│  │                 SQLite Pool (sqlx)                   │ │
│  │  • Connection management                             │ │
│  │  • Async queries                                     │ │
│  │  • Migration runner                                  │ │
│  └────────────────────────────┬─────────────────────────┘ │
└───────────────────────────────┬───────────────────────────┘
                                │
              ┌─────────────────▼────────────────────┐
              │        SQLite Database (file.db)     │
              │  ┌─────────────────────────────────┐ │
              │  │           sessions              │ │
              │  │  - id (PK)                      │ │
              │  │  - summary                      │ │
              │  │  - updated_at                   │ │
              │  └─────────────────────────────────┘ │
              │                                      │
              │  ┌─────────────────────────────────┐ │
              │  │             traces              │ │
              │  │  - id (PK)                      │ │
              │  │  - session_id (FK)              │ │
              │  │  - role (user/assistant/sys)    │ │
              │  │  - content (TEXT)               │ │
              │  │  - metadata (JSON)              │ │
              │  │      • duration_ms              │ │
              │  │      • success                  │ │
              │  │      • tokens_used              │ │
              │  │      • tools_called             │ │
              │  │  - created_at                   │ │
              │  │  - embedding (future)           │ │
              │  └─────────────────────────────────┘ │
              │                                      │
              │  ┌─────────────────────────────────┐ │
              │  │    traces_fts (FTS5 virtual)    │ │
              │  │  - role                         │ │
              │  │  - content                      │ │
              │  │  - metadata                     │ │
              │  │  + INSERT/UPDATE/DELETE triggers│ │
              │  └─────────────────────────────────┘ │
              └──────────────────────────────────────┘
```

  4. ⏳ Trace Lifecycle

```diagram
┌──────────────────────────────────────────────────────┐
│                 Message (from Rig)                   │
│  • role: String                                      │
│  • content: String                                   │
└───────────────────┬──────────────────────────────────┘
                    │
                    │ + metadata HashMap
                    │   (duration, success, tokens)
                    ▼
┌────────────────────────────────────────────────────────┐
│                     Trace::new()                       │
│  • Generate UUID                                       │
│  • Set timestamp                                       │
│  • Attach session_id                                   │
└───────────────────┬────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────┐
│                AgentHistory::log_trace()               │
│  • Serialize metadata to JSON                          │
│  • INSERT INTO traces                                  │
│  • FTS5 trigger auto-indexes                           │
└───────────────────┬────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────┐
│                   Persisted in SQLite                  │
│  • Queryable via FTS5 MATCH                            │
│  • Ranked by relevance                                 │
│  • Filterable by metadata                              │
└────────────────────────────────────────────────────────┘
```


## 🚙 How to Use

### Installation

Add `agentsmith` to your `Cargo.toml`:

```toml
[dependencies]
agentsmith = "0.1"
```

Or install with cargo:

```bash
cargo add agentsmith
```

### Basic Example

```rust
use agentsmith::{AgentHistory, SmartAgent};
use rig::providers::openai;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create persistent history
    let history = AgentHistory::new("my_agent.db", Some("session-123")).await?;

    // Create your Rig agent
    let client = openai::Client::new(&std::env::var("OPENAI_API_KEY")?);
    let agent = client
        .agent("gpt-4")
        .preamble("You are a helpful AI assistant with a perfect memory.")
        .build();

    // Wrap with SmartAgent for automatic memory
    let mut smart_agent = SmartAgent::new(agent, history);

    // Chat with automatic recall of relevant past interactions
    let reply = smart_agent.chat("How did we fix the JSON parsing bug?").await?;
    println!("Agent: {}", reply);

    Ok(())
}
```

### Advanced Usage

```rust
use agentsmith::{AgentHistory, Trace};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create history with custom session ID
    let history = AgentHistory::new("./chat.db", Some("debug-session")).await?;

    // Search past conversations
    let results = history.search("bug fix", 10, false).await?;
    for trace in results {
        println!("[{}] {}: {}",
            trace.created_at.format("%Y-%m-%d %H:%M:%S"),
            trace.role,
            trace.content
        );
    }

    // Get recent messages
    let recent = history.recent(5).await?;
    println!("Found {} recent messages", recent.len());

    // Manually add traces
    history.add_trace("user", "What's the status?").await?;
    history.add_trace("assistant", "All systems operational").await?;

    Ok(())
}
```

## 🧪 Examples

The repository includes several examples demonstrating different features:

```bash
# Persistent chat REPL that remembers across restarts
cargo run --example persistent_chat

# Demonstrate Atuin-style search capabilities
cargo run --example atuin_search_demo

# Import conversation logs from other sources
cargo run --example import_old_logs

# Basic usage example
cargo run --example basic
```

## 🧪 Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```

## 📚 Documentation

Comprehensive documentation is available at [docs.rs/agentsmith](https://docs.rs/agentsmith), including:
- API reference for all public types and functions
- Tutorial on wrapping Rig agents with persistent memory
- Examples of searching and managing conversation history
- Best practices for session management

## 🖊 Author

<a href="https://x.com/cryptopatrick">CryptoPatrick</a>

Keybase Verification:
https://keybase.io/cryptopatrick/sigs/8epNh5h2FtIX1UNNmf8YQ-k33M8J-Md4LnAN

## 🐣 Support
Leave a ⭐ if you think this project is cool.

## 🗄 License

This project is licensed under MIT. See [LICENSE](LICENSE) for details.
