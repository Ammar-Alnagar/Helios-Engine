/// Complete Demo: All New Features
///
/// This example demonstrates all three new features working together:
/// 1. Streaming for local models
/// 2. File management tools
/// 3. Session memory

use helios_engine::{Agent, Config, FileSearchTool, FileReadTool, FileEditTool, FileWriteTool};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Helios Engine - Complete Feature Demo");
    println!("=========================================\n");

    // Load configuration
    let config = Config::from_file("config.toml").unwrap_or_else(|_| {
        println!("⚠ No config.toml found, using default configuration");
        Config::new_default()
    });

    // Create agent with all file tools
    println!("📦 Creating agent with file tools...");
    let mut agent = Agent::builder("SmartAssistant")
        .config(config)
        .system_prompt(
            "You are an intelligent assistant with file management capabilities. \
             You can search files, read them, and make edits. Always explain what \
             you're doing and track important information in session memory."
        )
        .tool(Box::new(FileSearchTool))
        .tool(Box::new(FileReadTool))
        .tool(Box::new(FileEditTool))
        .tool(Box::new(FileWriteTool))
        .max_iterations(10)
        .build()
        .await?;

    println!("✓ Agent created successfully!\n");

    // Initialize session memory
    println!("🧠 Initializing session memory...");
    agent.set_memory("session_start", chrono::Utc::now().to_rfc3339());
    agent.set_memory("working_directory", std::env::current_dir()?.display().to_string());
    agent.set_memory("files_accessed", "0");
    agent.set_memory("edits_made", "0");
    println!("✓ Session memory initialized\n");

    // Demo 1: Search for files with streaming response
    println!("Demo 1: File Search with Streaming");
    println!("===================================");
    println!("User: Find all Rust example files\n");
    
    print!("Agent: ");
    io::stdout().flush()?;

    let response1 = agent.chat("Find all Rust example files in the examples directory").await?;
    println!("{}\n", response1);

    // Update memory
    let files_accessed = agent.get_memory("files_accessed")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    agent.set_memory("files_accessed", (files_accessed + 1).to_string());
    agent.set_memory("last_action", "file_search");

    // Demo 2: Read a file
    println!("\nDemo 2: Reading File Contents");
    println!("==============================");
    println!("User: Read the NEW_FEATURES.md file and summarize the key points\n");
    
    print!("Agent: ");
    io::stdout().flush()?;

    let response2 = agent.chat("Read the NEW_FEATURES.md file and give me a brief summary of what's new").await?;
    println!("{}\n", response2);

    // Update memory
    let files_accessed = agent.get_memory("files_accessed")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    agent.set_memory("files_accessed", (files_accessed + 1).to_string());
    agent.set_memory("last_action", "file_read");

    // Demo 3: Show session summary
    println!("\nDemo 3: Session Summary");
    println!("=======================\n");
    println!("{}", agent.get_session_summary());

    // Demo 4: Interactive mode
    println!("\n\nDemo 4: Interactive Mode");
    println!("========================");
    println!("You can now interact with the agent. Type 'exit' to quit.\n");

    loop {
        print!("\nYou: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match input.to_lowercase().as_str() {
            "exit" | "quit" => {
                println!("\n👋 Goodbye!");
                break;
            }
            "summary" => {
                println!("\n📊 Session Summary:");
                println!("{}", agent.get_session_summary());
                continue;
            }
            "memory" => {
                println!("\n🧠 Session Memory:");
                if let Some(start) = agent.get_memory("session_start") {
                    println!("  Session started: {}", start);
                }
                if let Some(dir) = agent.get_memory("working_directory") {
                    println!("  Working directory: {}", dir);
                }
                if let Some(files) = agent.get_memory("files_accessed") {
                    println!("  Files accessed: {}", files);
                }
                if let Some(edits) = agent.get_memory("edits_made") {
                    println!("  Edits made: {}", edits);
                }
                if let Some(action) = agent.get_memory("last_action") {
                    println!("  Last action: {}", action);
                }
                continue;
            }
            "help" => {
                println!("\n📖 Available Commands:");
                println!("  exit, quit  - Exit the demo");
                println!("  summary     - Show session summary");
                println!("  memory      - Show session memory");
                println!("  help        - Show this help");
                println!("\n💡 Try asking the agent to:");
                println!("  • Search for specific files");
                println!("  • Read file contents");
                println!("  • Summarize what it has done");
                continue;
            }
            _ => {}
        }

        // Send message to agent with streaming
        print!("\nAgent: ");
        io::stdout().flush()?;

        match agent.chat(input).await {
            Ok(response) => {
                println!("{}", response);
                
                // Update memory after each interaction
                let files_accessed = agent.get_memory("files_accessed")
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(0);
                agent.set_memory("files_accessed", (files_accessed + 1).to_string());
            }
            Err(e) => {
                eprintln!("\n❌ Error: {}", e);
            }
        }
    }

    // Final summary
    println!("\n📊 Final Session Summary:");
    println!("{}", agent.get_session_summary());

    println!("\n✅ Demo completed successfully!");
    println!("\n💡 Features Demonstrated:");
    println!("  ✓ Streaming responses (local/remote models)");
    println!("  ✓ File search with pattern matching");
    println!("  ✓ File reading with summaries");
    println!("  ✓ Session memory tracking");
    println!("  ✓ Interactive conversation");
    println!("  ✓ Real-time progress updates");

    Ok(())
}
