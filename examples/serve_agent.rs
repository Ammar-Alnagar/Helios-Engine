//! Example: Serving an agent via HTTP API
//!
//! This example demonstrates how to serve an agent using the Helios Engine serve feature.
//! The agent will be accessible via OpenAI-compatible API endpoints.

use helios_engine::{Agent, CalculatorTool, Config};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load configuration
    let config = Config::from_file("config.toml")?;

    // Create an agent with tools
    let agent = Agent::builder("API Agent")
        .config(config)
        .system_prompt("You are a helpful AI assistant with access to a calculator tool.")
        .tool(Box::new(CalculatorTool))
        .max_iterations(5)
        .build()
        .await?;

    // Start the server
    println!("Starting server on http://127.0.0.1:8000");
    println!("Try: curl http://127.0.0.1:8000/v1/chat/completions \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -d '{{\"model\": \"local-model\", \"messages\": [{{\"role\": \"user\", \"content\": \"What is 15 * 7?\"}}]}}'");

    helios_engine::serve::start_server_with_agent(
        agent,
        "local-model".to_string(),
        "127.0.0.1:8000",
    )
    .await?;

    Ok(())
}
