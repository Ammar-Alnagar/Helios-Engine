//! # Example: Agent with MemoryDB Tool
//!
//! This example demonstrates how to use the `MemoryDBTool` to provide an agent
//! with a simple in-memory key-value database. This allows the agent to cache
//! data, store user preferences, and maintain state across conversations.

use helios_engine::{Agent, Config, MemoryDBTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Helios Engine - Agent with Memory DB Example");
    println!("================================================\n");

    // Load configuration from `config.toml` or use default.
    let config = Config::from_file("config.toml").unwrap_or_else(|_| {
        println!("⚠ No config.toml found, using default configuration");
        Config::new_default()
    });

    // Create an agent named "DataAgent" and equip it with the MemoryDBTool.
    let mut agent = Agent::builder("DataAgent")
        .config(config)
        .system_prompt(
            "You are a helpful assistant with access to an in-memory database. \
             You can store and retrieve information using the memory_db tool. \
             Operations available: set, get, delete, list, clear, exists. \
             Use this to remember important information across our conversation.",
        )
        .tool(Box::new(MemoryDBTool::new()))
        .max_iterations(10)
        .build()
        .await?;

    println!("✓ Agent created with memory database tool\n");

    // --- Example 1: Store user preferences ---
    println!("Example 1: Storing User Preferences");
    println!("====================================\n");

    let response = agent
        .chat("Store my name as 'Alice' and my favorite color as 'blue' in the database")
        .await?;
    println!("Agent: {}\n", response);

    // --- Example 2: Retrieve stored data ---
    println!("\nExample 2: Retrieving Stored Data");
    println!("==================================\n");

    let response = agent.chat("What's my name and favorite color?").await?;
    println!("Agent: {}\n", response);

    // --- Example 3: Store calculations ---
    println!("\nExample 3: Caching Calculations");
    println!("================================\n");

    let response = agent
        .chat("Calculate 123 * 456 and store the result in the database with key 'calculation_result'")
        .await?;
    println!("Agent: {}\n", response);

    // --- Example 4: List all stored data ---
    println!("\nExample 4: Listing All Data");
    println!("===========================\n");

    let response = agent
        .chat("Show me everything stored in the database")
        .await?;
    println!("Agent: {}\n", response);

    // --- Example 5: Check if key exists ---
    println!("\nExample 5: Checking Key Existence");
    println!("==================================\n");

    let response = agent
        .chat("Check if 'name' and 'age' exist in the database")
        .await?;
    println!("Agent: {}\n", response);

    // --- Example 6: Delete specific data ---
    println!("\nExample 6: Deleting Data");
    println!("========================\n");

    let response = agent
        .chat("Delete the 'calculation_result' from the database")
        .await?;
    println!("Agent: {}\n", response);

    // --- Example 7: Final state ---
    println!("\nExample 7: Final Database State");
    println!("================================\n");

    let response = agent
        .chat("List all remaining items in the database")
        .await?;
    println!("Agent: {}\n", response);

    println!("\n✅ Example completed successfully!");
    println!("\n💡 Key Features Demonstrated:");
    println!("  • Setting key-value pairs in memory database");
    println!("  • Retrieving stored values");
    println!("  • Listing all database contents");
    println!("  • Checking key existence");
    println!("  • Deleting specific entries");
    println!("  • Persistent data across multiple agent interactions");
    println!("\n📝 Use Cases:");
    println!("  • Caching expensive computations");
    println!("  • Storing user preferences during conversation");
    println!("  • Maintaining context across multiple queries");
    println!("  • Temporary data storage for complex workflows");

    Ok(())
}
