# Serve Module

The `serve` module provides functionality to serve fully OpenAI-compatible API endpoints with real-time streaming and parameter control, allowing you to expose your agents or LLM clients via HTTP.

## Starting the Server

The `serve` module provides several functions for starting the server:

- **`start_server(config: Config, address: &str)`**: Starts the HTTP server with the given configuration.
- **`start_server_with_agent(agent: Agent, model_name: String, address: &str)`**: Starts the HTTP server with an agent.
- **`start_server_with_custom_endpoints(config: Config, address: &str, custom_endpoints: Option<CustomEndpointsConfig>)`**: Starts the HTTP server with custom endpoints.
- **`start_server_with_agent_and_custom_endpoints(agent: Agent, model_name: String, address: &str, custom_endpoints: Option<CustomEndpointsConfig>)`**: Starts the HTTP server with an agent and custom endpoints.

### Example

Here's an example of how to start the server with an agent:

```rust
use helios_engine::{Agent, Config, CalculatorTool, serve};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    let agent = Agent::builder("API Agent")
        .config(config)
        .system_prompt("You are a helpful AI assistant with access to a calculator tool.")
        .tool(Box::new(CalculatorTool))
        .max_iterations(5)
        .build()
        .await?;

    println!("Starting server on http://127.0.0.1:8000");
    println!("Try: curl http://127.0.0.1:8000/v1/chat/completions \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -d '{{\"model\": \"local-model\", \"messages\": [{{\"role\": \"user\", \"content\": \"What is 15 * 7?\"}}]}}'");

    serve::start_server_with_agent(agent, "local-model".to_string(), "127.0.0.1:8000").await?;

    Ok(())
}
```

## API Endpoints

The `serve` module exposes the following OpenAI-compatible API endpoints:

- **`POST /v1/chat/completions`**: Handles chat completion requests.
- **`GET /v1/models`**: Lists the available models.
- **`GET /health`**: A health check endpoint.

## Custom Endpoints

You can also define your own custom endpoints by creating a `custom_endpoints.toml` file and loading it when you start the server.

### `custom_endpoints.toml`

Here's an example of a `custom_endpoints.toml` file:

```toml
[[endpoints]]
method = "GET"
path = "/custom"
response = { message = "This is a custom endpoint" }
status_code = 200
```

### Loading Custom Endpoints

You can load the custom endpoints configuration using the `load_custom_endpoints_config()` function:

```rust
let custom_endpoints = serve::load_custom_endpoints_config("custom_endpoints.toml")?;
```

You can then pass the `custom_endpoints` to the `start_server_with_custom_endpoints()` or `start_server_with_agent_and_custom_endpoints()` function.
