//!
//! This example demonstrates the new simplified API for creating custom endpoints.
//! It's much easier and more intuitive than the old API!

use helios_engine::{Agent, CalculatorTool, Config, EndpointBuilder, ServerBuilder};

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

    // Create custom endpoints using the new simplified API
    println!("🎉 Creating custom endpoints with the new simplified API!\n");

    // Method 1: Super simple static endpoints
    let version_endpoint = helios_engine::get(
        "/api/version",
        serde_json::json!({
            "version": "0.4.4",
            "service": "Helios Engine",
            "features": ["agents", "tools", "streaming", "custom_endpoints"]
        }),
    );

    let status_endpoint = helios_engine::get(
        "/api/status",
        serde_json::json!({
            "status": "operational",
            "model": "agent-based"
        }),
    );

    // Method 2: Using the builder pattern for more control
    let info_endpoint = EndpointBuilder::get("/api/info")
        .json(serde_json::json!({
            "name": "Helios Engine API",
            "description": "AI Agent Server with Custom Endpoints",
            "documentation": "https://helios-engine.vercel.app/"
        }))
        .description("API information endpoint")
        .build();

    // Method 3: Dynamic responses with a handler function
    let echo_endpoint = EndpointBuilder::post("/api/echo")
        .handle(|req| {
            let message = req
                .and_then(|r| r.body)
                .and_then(|b| b.get("message").cloned())
                .unwrap_or_else(|| serde_json::json!("No message provided"));

            helios_engine::EndpointResponse::ok(serde_json::json!({
                "echo": message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        })
        .description("Echo endpoint that returns your message")
        .build();

    // Method 4: Different HTTP methods
    let create_endpoint = EndpointBuilder::post("/api/create")
        .json(serde_json::json!({
            "message": "Resource created",
            "id": "12345"
        }))
        .description("Simulates creating a resource")
        .build();

    let update_endpoint = EndpointBuilder::put("/api/update")
        .json(serde_json::json!({
            "message": "Resource updated"
        }))
        .description("Simulates updating a resource")
        .build();

    let delete_endpoint = helios_engine::delete(
        "/api/delete",
        serde_json::json!({
            "message": "Resource deleted"
        }),
    );

    // Collect all endpoints into a vector
    let custom_endpoints = vec![
        version_endpoint,
        status_endpoint,
        info_endpoint,
        echo_endpoint,
        create_endpoint,
        update_endpoint,
        delete_endpoint,
    ];

    // Start the server with the new ServerBuilder API
    println!("🚀 Starting server with custom endpoints...\n");
    println!("📡 OpenAI-compatible API endpoints:");
    println!("   POST /v1/chat/completions");
    println!("   GET  /v1/models");
    println!("\n📡 Custom endpoints:");
    println!("   GET    /api/version");
    println!("   GET    /api/status");
    println!("   GET    /api/info");
    println!("   POST   /api/echo");
    println!("   POST   /api/create");
    println!("   PUT    /api/update");
    println!("   DELETE /api/delete");
    println!("\n💡 Try these commands:");
    println!("   curl http://127.0.0.1:8000/api/version");
    println!("   curl http://127.0.0.1:8000/api/status");
    println!("   curl -X POST http://127.0.0.1:8000/api/echo \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{\"message\": \"Hello, Helios!\"}}'");
    println!();

    // Method 1: Pass a vector of endpoints (recommended)
    ServerBuilder::with_agent(agent, "local-model")
        .address("127.0.0.1:8000")
        .endpoints(custom_endpoints)
        .serve()
        .await?;

    // Method 2: Alternative - you can also use individual .endpoint() calls
    // ServerBuilder::with_agent(agent, "local-model")
    //     .address("127.0.0.1:8000")
    //     .endpoint(version_endpoint)
    //     .endpoint(status_endpoint)
    //     // ... etc
    //     .serve()
    //     .await?;

    Ok(())
}
