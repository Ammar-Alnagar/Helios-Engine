//! # Example: Agent with Tools
//!
//! This example demonstrates how to create an agent and equip it with tools.
//! The agent can then use these tools to perform tasks that it cannot do on its own.

use helios_engine::{Agent, CalculatorTool, Config, EchoTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Load configuration from `config.toml`.
    let config = Config::from_file("config.toml")?;

    // Create an agent named "ToolAgent" and equip it with the `CalculatorTool` and `EchoTool`.
    // Using the improved syntax to add multiple tools at once!
    let mut agent = Agent::builder("ToolAgent")
        .config(config)
        .system_prompt("You are a helpful assistant with access to tools. Use them when needed.")
        .tools(vec![Box::new(CalculatorTool), Box::new(EchoTool)])
        .max_iterations(5)
        .build()
        .await?;

    println!(
        "Available tools: {:?}\n",
        agent.tool_registry().list_tools()
    );

    // --- Test the calculator tool ---
    let response = agent.chat("What is 25 * 4 + 10?").await?;
    println!("Agent: {}\n", response);

    // --- Test the echo tool ---
    let response = agent
        .chat("Can you echo this message: 'Hello from Helios!'")
        .await?;
    println!("Agent: {}\n", response);

    Ok(())
}
