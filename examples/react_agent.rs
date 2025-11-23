//! # Example: ReAct Agent
//!
//! This example demonstrates the ReAct (Reasoning and Acting) feature.
//! When enabled, the agent will reason about the task and create a plan
//! before taking actions, leading to more thoughtful and systematic problem-solving.

use helios_engine::{Agent, CalculatorTool, Config, EchoTool, FileReadTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🧠 Helios Engine - ReAct Agent Example");
    println!("======================================\n");

    // Load configuration from `config.toml`.
    let config = Config::from_file("config.toml")?;

    // Create an agent with ReAct mode enabled.
    // Notice the simple `.react()` call in the builder pattern!
    let mut agent = Agent::builder("ReActAgent")
        .config(config)
        .system_prompt(
            "You are a helpful assistant that thinks carefully before acting. \
             Use your reasoning to plan your approach.",
        )
        .tools(vec![
            Box::new(CalculatorTool),
            Box::new(EchoTool),
            Box::new(FileReadTool),
        ])
        .react() // Enable ReAct mode - that's all it takes!
        .max_iterations(5)
        .build()
        .await?;

    println!(
        "Available tools: {:?}\n",
        agent.tool_registry().list_tools()
    );

    println!("═══════════════════════════════════════════════════════════");
    println!("Example 1: Mathematical Problem");
    println!("═══════════════════════════════════════════════════════════\n");

    // --- Test 1: Math problem requiring reasoning ---
    println!("User: I need to calculate (25 * 4) + (100 / 5). Can you help?\n");
    let response = agent
        .chat("I need to calculate (25 * 4) + (100 / 5). Can you help?")
        .await?;
    println!("\nAgent: {}\n", response);

    println!("═══════════════════════════════════════════════════════════");
    println!("Example 2: Multi-step Task");
    println!("═══════════════════════════════════════════════════════════\n");

    // --- Test 2: Multi-step task ---
    println!("User: First calculate 15 * 7, then echo the result back to me.\n");
    let response = agent
        .chat("First calculate 15 * 7, then echo the result back to me.")
        .await?;
    println!("\nAgent: {}\n", response);

    println!("═══════════════════════════════════════════════════════════");
    println!("✅ ReAct Demo Complete!");
    println!("═══════════════════════════════════════════════════════════");
    println!("\nNotice how the agent:");
    println!("  1. 💭 First reasons about the task");
    println!("  2. 📋 Creates a plan");
    println!("  3. ⚡ Then executes the actions\n");
    println!("This leads to more thoughtful and systematic problem-solving!");

    Ok(())
}
