//! # Example: Basic Chat
//!
//! This example demonstrates the simplest way to use the Helios Engine: a basic chat
//! with an agent. The agent maintains a conversation history and can respond to
//! multiple turns of conversation.

use helios_engine::{Agent, Config};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Load configuration from `config.toml`.
    let config = Config::from_file("config.toml")?;

    // Create a simple agent named "BasicAgent".
    let mut agent = Agent::builder("BasicAgent")
        .config(config)
        .system_prompt("You are a helpful assistant.")
        .build()
        .await?;

    // --- Send a message to the agent ---
    let response = agent.chat("Hello! How are you?").await?;
    println!("Agent: {}", response);

    // --- Continue the conversation ---
    let response = agent.chat("What can you help me with?").await?;
    println!("Agent: {}", response);

    Ok(())
}
