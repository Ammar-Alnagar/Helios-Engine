//! Example: Serving with Custom Endpoints
//!
//! This example demonstrates how to serve an agent with custom endpoints
//! in addition to the standard OpenAI-compatible API.

use helios_engine::{Agent, CalculatorTool, Config, CustomEndpoint, CustomEndpointsConfig};

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

    // Define custom endpoints
    let custom_endpoints = CustomEndpointsConfig {
        endpoints: vec![
            CustomEndpoint {
                method: "GET".to_string(),
                path: "/api/version".to_string(),
                response: serde_json::json!({
                    "version": "0.2.8",
                    "service": "Helios Engine",
                    "features": ["agents", "tools", "streaming", "custom_endpoints"]
                }),
                status_code: 200,
            },
            CustomEndpoint {
                method: "GET".to_string(),
                path: "/api/status".to_string(),
                response: serde_json::json!({
                    "status": "operational",
                    "uptime": "unknown",
                    "model": "agent-based"
                }),
                status_code: 200,
            },
            CustomEndpoint {
                method: "POST".to_string(),
                path: "/api/echo".to_string(),
                response: serde_json::json!({
                    "message": "Echo endpoint - this returns static data",
                    "note": "For dynamic responses, use the chat completions endpoint"
                }),
                status_code: 200,
            },
        ],
    };

    // Start the server with custom endpoints
    println!("Starting server on http://127.0.0.1:8000");
    println!("📡 OpenAI-compatible API endpoints:");
    println!("   POST /v1/chat/completions");
    println!("   GET  /v1/models");
    println!("📡 Custom endpoints:");
    println!("   GET  /api/version");
    println!("   GET  /api/status");
    println!("   POST /api/echo");
    println!();
    println!("Try: curl http://127.0.0.1:8000/api/version");

    helios_engine::serve::start_server_with_agent_and_custom_endpoints(
        agent,
        "local-model".to_string(),
        "127.0.0.1:8000",
        Some(custom_endpoints),
    )
    .await?;

    Ok(())
}
