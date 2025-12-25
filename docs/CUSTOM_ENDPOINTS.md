# Custom Endpoints Guide

This guide explains how to create custom HTTP endpoints for your Helios Engine server using the new simplified API.

## Overview

Helios Engine allows you to add custom HTTP endpoints to your agent server, enabling you to expose additional functionality beyond the standard OpenAI-compatible API. The new endpoint builder API makes this incredibly simple and intuitive.

## Quick Start

Here's the simplest way to create a custom endpoint:

```rust
use helios_engine::{ServerBuilder, get};

// Create a simple GET endpoint
let endpoint = get("/api/status", serde_json::json!({
    "status": "ok",
    "uptime": "24h"
}));

// Add it to your server
ServerBuilder::with_agent(agent, "model-name")
    .endpoint(endpoint)
    .serve()
    .await?;
```

## Creating Endpoints

### Method 1: Simple Helper Functions (Recommended for Static Data)

The easiest way to create endpoints with static JSON responses:

```rust
use helios_engine::{get, post, put, delete, patch};

// GET endpoint
let version = get("/api/version", serde_json::json!({
    "version": "1.0.0",
    "service": "My API"
}));

// POST endpoint
let create = post("/api/create", serde_json::json!({
    "message": "Resource created"
}));

// PUT endpoint
let update = put("/api/update", serde_json::json!({
    "message": "Resource updated"
}));

// DELETE endpoint
let remove = delete("/api/delete", serde_json::json!({
    "message": "Resource deleted"
}));

// PATCH endpoint
let patch_resource = patch("/api/patch", serde_json::json!({
    "message": "Resource patched"
}));
```

### Method 2: Builder Pattern (For More Control)

Use the builder pattern when you need additional features:

```rust
use helios_engine::EndpointBuilder;

let endpoint = EndpointBuilder::get("/api/info")
    .json(serde_json::json!({
        "name": "My API",
        "description": "A helpful API"
    }))
    .description("API information endpoint")
    .build();
```

### Method 3: Dynamic Handlers (For Interactive Endpoints)

Create endpoints that respond dynamically to request data:

```rust
use helios_engine::{EndpointBuilder, EndpointResponse};

let echo = EndpointBuilder::post("/api/echo")
    .handle(|req| {
        // Extract message from request body
        let message = req
            .and_then(|r| r.body)
            .and_then(|b| b.get("message").cloned())
            .unwrap_or_else(|| serde_json::json!("No message"));

        // Return a dynamic response
        EndpointResponse::ok(serde_json::json!({
            "echo": message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    })
    .description("Echo your message back")
    .build();
```

### Method 4: Using Response Helpers

Create responses with appropriate HTTP status codes:

```rust
use helios_engine::{EndpointBuilder, EndpointResponse};

let find_user = EndpointBuilder::get("/api/users/:id")
    .handle(|req| {
        // Check if user exists (simplified example)
        let user_exists = true;
        
        if user_exists {
            EndpointResponse::ok(serde_json::json!({
                "id": "123",
                "name": "John Doe"
            }))
        } else {
            EndpointResponse::not_found("User not found")
        }
    })
    .build();
```

Available response helpers:
- `EndpointResponse::ok(body)` - 200 OK
- `EndpointResponse::created(body)` - 201 CREATED
- `EndpointResponse::accepted(body)` - 202 ACCEPTED
- `EndpointResponse::bad_request(message)` - 400 BAD REQUEST
- `EndpointResponse::not_found(message)` - 404 NOT FOUND
- `EndpointResponse::internal_error(message)` - 500 INTERNAL SERVER ERROR
- `EndpointResponse::with_status(status, body)` - Custom status code

## Adding Endpoints to Your Server

### Option 1: Using a Vector (Recommended)

The cleanest way to add multiple endpoints:

```rust
use helios_engine::{ServerBuilder, get, post};

let endpoints = vec![
    get("/api/version", serde_json::json!({"version": "1.0"})),
    get("/api/status", serde_json::json!({"status": "ok"})),
    post("/api/echo", serde_json::json!({"message": "echo"})),
];

ServerBuilder::with_agent(agent, "model-name")
    .address("127.0.0.1:8000")
    .endpoints(endpoints)
    .serve()
    .await?;
```

### New Simplified API Benefits

The new simplified API offers several advantages over the old approach:

**New API (Recommended):**
```rust
use helios_engine::{ServerBuilder, get, post, EndpointBuilder, EndpointResponse};

let endpoints = vec![
    // Simple static endpoint
    get("/api/version", serde_json::json!({
        "version": "1.0.0",
        "service": "Helios Engine"
    })),

    // Dynamic endpoint with handler
    EndpointBuilder::post("/api/echo")
        .handle(|req| {
            let message = req
                .and_then(|r| r.body)
                .and_then(|b| b.get("message").cloned())
                .unwrap_or_else(|| serde_json::json!("No message"));

            EndpointResponse::ok(serde_json::json!({
                "echo": message
            }))
        })
        .build(),
];

ServerBuilder::with_agent(agent, "local-model")
    .address("127.0.0.1:8000")
    .endpoints(endpoints)
    .serve()
    .await?;
```

**Benefits of the New API:**
1. **Simpler syntax** - Less boilerplate code
2. **Builder pattern** - Consistent with Agent API
3. **Dynamic handlers** - Process request data and return dynamic responses
4. **Type safety** - Better compile-time guarantees
5. **Helper functions** - Quick creation with `get()`, `post()`, etc.
6. **Response helpers** - Easy status code management
7. **Descriptions** - Document your endpoints inline
8. **Cleaner** - Vector-based endpoint management
```

### Option 2: Using Array Slice Syntax

Pass endpoints inline without creating a variable:

```rust
ServerBuilder::with_agent(agent, "model-name")
    .with_endpoints(&[
        get("/api/v1", serde_json::json!({"version": "1.0"})),
        get("/api/status", serde_json::json!({"status": "ok"})),
    ])
    .serve()
    .await?;
```

### Option 3: Adding Endpoints One by One

If you prefer a fluent API style:

```rust
ServerBuilder::with_agent(agent, "model-name")
    .endpoint(get("/api/version", serde_json::json!({"version": "1.0"})))
    .endpoint(get("/api/status", serde_json::json!({"status": "ok"})))
    .serve()
    .await?;
```

## Complete Example

Here's a complete example showing various endpoint types:

```rust
use helios_engine::{
    Agent, Config, CalculatorTool, ServerBuilder,
    EndpointBuilder, EndpointResponse, get, post
};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Create agent
    let config = Config::from_file("config.toml")?;
    let agent = Agent::builder("API Agent")
        .config(config)
        .tool(Box::new(CalculatorTool))
        .build()
        .await?;

    // Define custom endpoints
    let endpoints = vec![
        // Simple static endpoint
        get("/api/version", serde_json::json!({
            "version": "1.0.0",
            "service": "My API"
        })),
        
        // Endpoint with description
        EndpointBuilder::get("/api/health")
            .json(serde_json::json!({"status": "healthy"}))
            .description("Health check endpoint")
            .build(),
        
        // Dynamic endpoint with handler
        EndpointBuilder::post("/api/greet")
            .handle(|req| {
                let name = req
                    .and_then(|r| r.body)
                    .and_then(|b| b.get("name").and_then(|n| n.as_str()))
                    .unwrap_or("World");
                
                EndpointResponse::ok(serde_json::json!({
                    "message": format!("Hello, {}!", name)
                }))
            })
            .description("Personalized greeting")
            .build(),
    ];

    // Start server
    ServerBuilder::with_agent(agent, "gpt-4")
        .address("127.0.0.1:8000")
        .endpoints(endpoints)
        .serve()
        .await?;

    Ok(())
}
```

## Testing Your Endpoints

### Using curl

```bash
# Test GET endpoint
curl http://127.0.0.1:8000/api/version

# Test POST endpoint with JSON body
curl -X POST http://127.0.0.1:8000/api/greet \
  -H 'Content-Type: application/json' \
  -d '{"name": "Alice"}'

# Test the standard chat endpoint
curl -X POST http://127.0.0.1:8000/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### Using HTTPie

```bash
# Test GET endpoint
http GET http://127.0.0.1:8000/api/version

# Test POST endpoint
http POST http://127.0.0.1:8000/api/greet name=Alice
```

## Request Data Access

Dynamic handlers receive an `Option<EndpointRequest>` with access to:

```rust
pub struct EndpointRequest {
    /// Query parameters from URL (?key=value)
    pub query: HashMap<String, String>,
    
    /// Path parameters (/users/:id)
    pub params: HashMap<String, String>,
    
    /// Request body as JSON (for POST, PUT, PATCH)
    pub body: Option<Value>,
}
```

Example using request data:

```rust
EndpointBuilder::post("/api/process")
    .handle(|req| {
        let req = req.unwrap();
        
        // Access query parameters
        let limit = req.query.get("limit").cloned();
        
        // Access path parameters
        let id = req.params.get("id").cloned();
        
        // Access request body
        let data = req.body.unwrap_or(serde_json::json!({}));
        
        EndpointResponse::ok(serde_json::json!({
            "received": data,
            "limit": limit,
            "id": id
        }))
    })
    .build()
```

## Comparison: Old vs New API

### Old API (Still Supported for Backward Compatibility)

```rust
use helios_engine::{CustomEndpoint, CustomEndpointsConfig};

let custom_endpoints = CustomEndpointsConfig {
    endpoints: vec![
        CustomEndpoint {
            method: "GET".to_string(),
            path: "/api/version".to_string(),
            response: serde_json::json!({
                "version": "1.0.0",
                "service": "Helios Engine"
            }),
            status_code: 200,
        },
        CustomEndpoint {
            method: "POST".to_string(),
            path: "/api/echo".to_string(),
            response: serde_json::json!({
                "message": "Static response only"
            }),
            status_code: 200,
        },
    ],
};

helios_engine::serve::start_server_with_agent_and_custom_endpoints(
    agent,
    "local-model".to_string(),
    "127.0.0.1:8000",
    Some(custom_endpoints),
).await?;
```

### New API (Recommended)

```rust
use helios_engine::{ServerBuilder, get, post, EndpointBuilder, EndpointResponse};

let endpoints = vec![
    // Simple static endpoint
    get("/api/version", serde_json::json!({
        "version": "1.0.0",
        "service": "Helios Engine"
    })),
    
    // Dynamic endpoint with handler
    EndpointBuilder::post("/api/echo")
        .handle(|req| {
            let message = req
                .and_then(|r| r.body)
                .and_then(|b| b.get("message").cloned())
                .unwrap_or_else(|| serde_json::json!("No message"));
            
            EndpointResponse::ok(serde_json::json!({
                "echo": message
            }))
        })
        .build(),
];

ServerBuilder::with_agent(agent, "local-model")
    .address("127.0.0.1:8000")
    .endpoints(endpoints)
    .serve()
    .await?;
```

### Benefits of the New API

1.  **Simpler syntax** - Less boilerplate code
2.  **Builder pattern** - Consistent with Agent API
3.  **Dynamic handlers** - Process request data and return dynamic responses
4.  **Type safety** - Better compile-time guarantees
5.  **Helper functions** - Quick creation with `get()`, `post()`, etc.
6.  **Response helpers** - Easy status code management
7.  **Descriptions** - Document your endpoints inline
8.  **Cleaner** - Vector-based endpoint management

## Best Practices

1. **Use vectors for multiple endpoints** - It's cleaner and more maintainable
2. **Add descriptions** - Help document your API
3. **Use response helpers** - They make status codes clear and consistent
4. **Validate input** - Always check request data before processing
5. **Handle errors gracefully** - Return appropriate error responses

## Migration Guide

If you're using the old API, here's how to migrate:

**Before:**
```rust
CustomEndpoint {
    method: "GET".to_string(),
    path: "/api/status".to_string(),
    response: serde_json::json!({"status": "ok"}),
    status_code: 200,
}
```

**After:**
```rust
get("/api/status", serde_json::json!({"status": "ok"}))
```

That's it! The new API is much simpler and more powerful.

## See Also

- [API Reference](API.md) - Complete API documentation
- [Examples](../examples/) - More code examples
- [Server Guide](GETTING_STARTED.md#serving-agents) - Server setup and configuration
