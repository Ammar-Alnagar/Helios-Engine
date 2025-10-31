//! # Example: Agent with File Tools
//!
//! This example demonstrates how to create an agent with file manipulation tools,
//! including `FileSearchTool`, `FileReadTool`, `FileEditTool`, and `FileWriteTool`.
//! It also shows how to use session memory to track the agent's state.

use helios_engine::{Agent, Config, FileEditTool, FileReadTool, FileSearchTool, FileWriteTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Helios Engine - Agent with File Tools Example");
    println!("=================================================\n");

    // Load configuration
    let config = Config::from_file("config.toml").unwrap_or_else(|_| {
        println!("⚠ No config.toml found, using default configuration");
        Config::new_default()
    });

    // Create an agent named "FileAssistant" and equip it with file tools.
    let mut agent = Agent::builder("FileAssistant")
        .config(config)
        .system_prompt(
            "You are a helpful file management assistant. You can search for files, \
             read file contents, and edit files. Always confirm with the user before \
             making changes to files. Keep track of important session information.",
        )
        .tool(Box::new(FileSearchTool))
        .tool(Box::new(FileReadTool))
        .tool(Box::new(FileEditTool))
        .tool(Box::new(FileWriteTool))
        .max_iterations(10)
        .build()
        .await?;

    println!("✓ Agent created with file tools");
    println!("✓ Available tools: file_search, file_read, file_edit, file_write\n");

    // Set initial session memory for the agent.
    agent.set_memory("session_start", chrono::Utc::now().to_rfc3339());
    agent.set_memory(
        "working_directory",
        std::env::current_dir()?.display().to_string(),
    );
    agent.set_memory("tasks_completed", "0");

    // --- Example 1: Search for Rust files ---
    println!("Example 1: Searching for Rust files");
    println!("====================================\n");

    let response = agent
        .chat("Find all Rust source files in the src directory")
        .await?;
    println!("Agent: {}\n", response);

    // Update session memory after the task.
    agent.increment_tasks_completed();
    agent.set_memory("last_task", "file_search");

    // --- Example 2: Read a specific file ---
    println!("\nExample 2: Reading file contents");
    println!("==================================\n");

    let response = agent
        .chat("Read the contents of src/lib.rs and give me a summary")
        .await?;
    println!("Agent: {}\n", response);

    // Update session memory after the task.
    agent.increment_tasks_completed();
    agent.set_memory("last_task", "file_read");

    // --- Example 3: Show session summary ---
    println!("\nExample 3: Session Summary");
    println!("==========================\n");

    println!("{}", agent.get_session_summary());

    // --- Example 4: Check session memory ---
    println!("\nExample 4: Checking Session Memory");
    println!("===================================\n");

    println!(
        "Working directory: {}",
        agent
            .get_memory("working_directory")
            .unwrap_or(&"unknown".to_string())
    );
    println!(
        "Tasks completed: {}",
        agent
            .get_memory("tasks_completed")
            .unwrap_or(&"0".to_string())
    );
    println!(
        "Last task: {}",
        agent.get_memory("last_task").unwrap_or(&"none".to_string())
    );

    println!("\n✅ Example completed successfully!");
    println!("\n💡 Key Features Demonstrated:");
    println!("  • File search with pattern matching and content search");
    println!("  • File reading with line range support");
    println!("  • File editing with find/replace functionality");
    println!("  • Session memory for tracking agent state");
    println!("  • Streaming responses (works with both local and remote models)");

    Ok(())
}
