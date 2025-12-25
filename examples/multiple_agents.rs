//! # Example: Multiple Agents
//!
//! This example demonstrates how to create and use multiple agents with different
//! personalities and tools. Each agent maintains its own separate conversation
//! history and can be specialized for different tasks.

use helios_engine::{Agent, CalculatorTool, Config};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Load configuration from `config.toml`.
    let config = Config::from_file("config.toml")?;

    // --- Create multiple agents with different personalities and tools ---

    // An agent specialized in math, equipped with a calculator tool.
    let mut math_agent = Agent::builder("MathAgent")
        .config(config.clone())
        .system_prompt("You are a math expert. You love numbers and equations.")
        .tool(Box::new(CalculatorTool))
        .build()
        .await?;

    // A creative agent for writing and storytelling.
    let mut creative_agent = Agent::builder("CreativeAgent")
        .config(config)
        .system_prompt("You are a creative writer who loves storytelling and poetry.")
        .build()
        .await?;

    // --- Interact with the Math Agent ---
    println!("=== Math Agent ===");
    let response = math_agent.chat("What is the square root of 144?").await?;
    println!("Math Agent: {}\n", response);

    // --- Interact with the Creative Agent ---
    println!("=== Creative Agent ===");
    let response = creative_agent
        .chat("Write a haiku about programming.")
        .await?;
    println!("Creative Agent: {}\n", response);

    Ok(())
}
