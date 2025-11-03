//! # Example: Basic Chat with Real-Time Streaming
//!
//! This example demonstrates the simplest way to use the Helios Engine: a basic chat
//! with an agent. The agent maintains a conversation history and can respond to
//! multiple turns of conversation.
//!
//! **NEW**: Streaming is now enabled by default! Watch as responses appear token-by-token
//! in real-time, just like ChatGPT's interface.

use helios_engine::{Agent, Config};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Helios Engine - Basic Chat Example");
    println!("=====================================");
    println!("💡 Streaming is enabled by default - watch tokens appear in real-time!\n");

    // Load configuration from `config.toml`.
    let config = Config::from_file("config.toml")?;

    // Create a simple agent named "BasicAgent".
    let mut agent = Agent::builder("BasicAgent")
        .config(config)
        .system_prompt("You are a helpful assistant.")
        .build()
        .await?;

    // --- Send a message to the agent ---
    println!("User: Hello! How are you?");
    print!("Agent (streaming): ");
    io::stdout().flush()?;

    let _response = agent.chat("Hello! How are you?").await?;
    println!();

    // --- Continue the conversation ---
    println!("\nUser: What can you help me with?");
    print!("Agent (streaming): ");
    io::stdout().flush()?;

    let _response = agent.chat("What can you help me with?").await?;
    println!();

    println!("\n✅ Demo completed! Notice how responses streamed in real-time.");

    Ok(())
}
