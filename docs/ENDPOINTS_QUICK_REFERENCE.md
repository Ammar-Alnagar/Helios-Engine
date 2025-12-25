# Custom Endpoints Quick Reference

A cheat sheet for creating custom HTTP endpoints in Helios Engine.

## Basic Imports

```rust
use helios_engine::{
    ServerBuilder,                    // Server builder
    get, post, put, delete, patch,   // Quick helpers
    EndpointBuilder,                  // Advanced builder
    EndpointResponse,                 // Response helpers
    EndpointRequest,                  // Request data
};
```

## Quick Examples

### 1. Simple Static Endpoint (One-liner)

```rust
get("/api/status", serde_json::json!({"status": "ok"}))
```

### 2. Multiple Static Endpoints

```rust
let endpoints = vec![
    get("/api/version", serde_json::json!({"version": "1.0"})),
    post("/api/ping", serde_json::json!({"pong": true})),
    put("/api/update", serde_json::json!({"updated": true})),
    delete("/api/delete", serde_json::json!({"deleted": true})),
];
```

### 3. Dynamic Endpoint with Handler

```rust
EndpointBuilder::post("/api/echo")
    .handle(|req| {
        let msg = req.and_then(|r| r.body).unwrap_or_default();
        EndpointResponse::ok(serde_json::json!({"echo": msg}))
    })
    .build()
```

### 4. Endpoint with Description

```rust
EndpointBuilder::get("/api/health")
    .json(serde_json::json!({"status": "healthy"}))
    .description("Health check endpoint")
    .build()
```

### 5. Starting the Server

```rust
ServerBuilder::with_agent(agent, "model-name")
    .address("127.0.0.1:8000")
    .endpoints(endpoints)
    .serve()
    .await?;
```

## HTTP Methods

| Method | Helper Function | Builder Method |
|--------|----------------|----------------|
| GET | `get(path, json)` | `EndpointBuilder::get(path)` |
| POST | `post(path, json)` | `EndpointBuilder::post(path)` |
| PUT | `put(path, json)` | `EndpointBuilder::put(path)` |
| DELETE | `delete(path, json)` | `EndpointBuilder::delete(path)` |
| PATCH | `patch(path, json)` | `EndpointBuilder::patch(path)` |

## Response Helpers

```rust
// 200 OK
EndpointResponse::ok(json!({"data": "value"}))

// 201 CREATED
EndpointResponse::created(json!({"id": "123"}))

// 202 ACCEPTED
EndpointResponse::accepted(json!({"queued": true}))

// 400 BAD REQUEST
EndpointResponse::bad_request("Invalid input")

// 404 NOT FOUND
EndpointResponse::not_found("Resource not found")

// 500 INTERNAL SERVER ERROR
EndpointResponse::internal_error("Something went wrong")

// Custom status code
EndpointResponse::with_status(StatusCode::CREATED, json!({"id": 1}))
```

## Request Data Access

```rust
EndpointBuilder::post("/api/process")
    .handle(|req| {
        let req = req.unwrap();
        
        // Query parameters (?key=value)
        let query = &req.query;
        let limit = query.get("limit");
        
        // Path parameters (/users/:id)
        let params = &req.params;
        let id = params.get("id");
        
        // Request body (JSON)
        let body = req.body.unwrap_or_default();
        let name = body.get("name");
        
        EndpointResponse::ok(json!({"processed": true}))
    })
    .build()
```

## Common Patterns

### Validation

```rust
EndpointBuilder::post("/api/users")
    .handle(|req| {
        let body = req.and_then(|r| r.body).unwrap_or_default();
        
        match body.get("email") {
            Some(email) if email.is_string() => {
                EndpointResponse::created(json!({"user": "created"}))
            }
            _ => EndpointResponse::bad_request("Email is required")
        }
    })
    .build()
```

### Conditional Responses

```rust
EndpointBuilder::get("/api/resource/:id")
    .handle(|req| {
        let exists = check_if_exists();
        
        if exists {
            EndpointResponse::ok(json!({"data": "..."}))
        } else {
            EndpointResponse::not_found("Not found")
        }
    })
    .build()
```

### Error Handling

```rust
EndpointBuilder::post("/api/process")
    .handle(|req| {
        match process_request(req) {
            Ok(result) => EndpointResponse::ok(json!(result)),
            Err(e) => EndpointResponse::internal_error(&e.to_string())
        }
    })
    .build()
```

## Server Builder Options

```rust
ServerBuilder::with_agent(agent, "model-name")
    .address("127.0.0.1:8000")        // Server address
    .endpoint(single_endpoint)         // Add one endpoint
    .endpoints(vec![...])              // Add multiple endpoints
    .with_endpoints(&[...])            // Add from array slice
    .serve()                           // Start the server
    .await?;
```

## Complete Example Template

```rust
use helios_engine::{
    Agent, Config, ServerBuilder,
    get, EndpointBuilder, EndpointResponse,
};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // 1. Create agent
    let config = Config::from_file("config.toml")?;
    let agent = Agent::builder("MyAgent")
        .config(config)
        .build()
        .await?;
    
    // 2. Define endpoints
    let endpoints = vec![
        // Static endpoint
        get("/api/info", serde_json::json!({
            "name": "My API",
            "version": "1.0"
        })),
        
        // Dynamic endpoint
        EndpointBuilder::post("/api/echo")
            .handle(|req| {
                let msg = req.and_then(|r| r.body).unwrap_or_default();
                EndpointResponse::ok(serde_json::json!({"echo": msg}))
            })
            .build(),
    ];
    
    // 3. Start server
    ServerBuilder::with_agent(agent, "model")
        .address("127.0.0.1:8000")
        .endpoints(endpoints)
        .serve()
        .await?;
    
    Ok(())
}
```

## Testing Commands

```bash
# Test GET endpoint
curl http://127.0.0.1:8000/api/status

# Test POST endpoint
curl -X POST http://127.0.0.1:8000/api/echo \
  -H 'Content-Type: application/json' \
  -d '{"message": "Hello!"}'

# Test with query parameters
curl http://127.0.0.1:8000/api/search?q=rust&limit=10

# Test the agent endpoint
curl -X POST http://127.0.0.1:8000/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "model-name",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

## Tips & Best Practices

1.  **Use vectors** for multiple endpoints - it's cleaner
2.  **Add descriptions** to document your endpoints
3.  **Use response helpers** for consistent status codes
4.  **Validate input** before processing
5.  **Handle errors gracefully** with appropriate responses
6.  **Keep handlers simple** - delegate complex logic to functions
7.  **Use static helpers** (`get`, `post`, etc.) for simple cases
8.  **Use builder pattern** when you need more control

## See Also

- [CUSTOM_ENDPOINTS.md](CUSTOM_ENDPOINTS.md) - Complete guide
- [CUSTOM_ENDPOINTS_MIGRATION.md](CUSTOM_ENDPOINTS_MIGRATION.md) - Migration from old API
- [examples/serve_simple_endpoints.rs](../examples/serve_simple_endpoints.rs) - Working example
