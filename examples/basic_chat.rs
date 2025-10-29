use helios::{Agent, Config};

#[tokio::main]
async fn main() -> helios::Result<()> {
    // Load configuration
    let config = Config::from_file("config.toml")?;

    // Create a simple agent
    let mut agent = Agent::builder("BasicAgent")
        .config(config)
        .system_prompt("You are a helpful assistant.")
        .build()?;

    // Send a message
    let response = agent.chat("Hello! How are you?").await?;
    println!("Agent: {}", response);

    // Continue the conversation
    let response = agent.chat("What can you help me with?").await?;
    println!("Agent: {}", response);

    Ok(())
}
