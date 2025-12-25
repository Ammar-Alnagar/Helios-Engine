//! # Quick Start Example
//!
//! Get started with Helios in seconds!
//! This is the fastest way to create and use an AI agent.

use helios_engine::Agent;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("⚡ Helios Quick Start\n");

    // Step 1: Create an agent in ONE LINE (using auto config)
    let mut agent = Agent::quick("MyAgent").await?;
    println!("✓ Agent created");

    // Step 2: Ask a question
    let response = agent.ask("What's the capital of France?").await?;
    println!("Q: What's the capital of France?");
    println!("A: {}\n", response);

    // Step 3: Ask another question (agent remembers context!)
    let response2 = agent.ask("What's its population?").await?;
    println!("Q: What's its population?");
    println!("A: {}\n", response2);

    // That's it! You're using Helios! 🎉
    println!("✅ Done! Create an agent and chat - that simple!");

    Ok(())
}
