# Custom Endpoints Migration Guide

## Overview

Helios Engine v0.4.5+ introduces a completely redesigned API for creating custom HTTP endpoints. The new API is **simpler, more powerful, and more intuitive** than the previous version.

## Quick Comparison

### Before (Old API) ❌
```rust
use helios_engine::{CustomEndpoint, CustomEndpointsConfig};

let custom_endpoints = CustomEndpointsConfig {
    endpoints: vec![
        CustomEndpoint {
            method: "GET".to_string(),
            path: "/api/version".to_string(),
            response: serde_json::json!({"version": "1.0.0"}),
            status_code: 200,
        },
        CustomEndpoint {
            method: "POST".to_string(),
            path: "/api/echo".to_string(),
            response: serde_json::json!({"message": "Static only"}),
            status_code: 200,
        },
    ],
};

helios_engine::serve::start_server_with_agent_and_custom_endpoints(
    agent,
    "model-name".to_string(),
    "127.0.0.1:8000",
    Some(custom_endpoints),
).await?;
```

**Problems:**
- ❌ Too verbose (manual struct creation)
- ❌ Only supports static responses
- ❌ Can't access request data
- ❌ Long function names
- ❌ Inconsistent with Agent builder API

### After (New API) 
```rust
use helios_engine::{ServerBuilder, get, post, EndpointBuilder, EndpointResponse};

let endpoints = vec![
    get("/api/version", serde_json::json!({"version": "1.0.0"})),
    
    EndpointBuilder::post("/api/echo")
        .handle(|req| {
            let msg = req.and_then(|r| r.body).unwrap_or_default();
            EndpointResponse::ok(serde_json::json!({"echo": msg}))
        })
        .build(),
];

ServerBuilder::with_agent(agent, "model-name")
    .address("127.0.0.1:8000")
    .endpoints(endpoints)
    .serve()
    .await?;
```

**Benefits:**
-  Clean and concise
-  Supports dynamic responses
-  Access to request data (body, query, params)
-  Builder pattern (consistent with Agent API)
-  Helper functions for common cases
-  Response status helpers

## Migration Steps

### Step 1: Update Imports

**Old:**
```rust
use helios_engine::{
    start_server_with_agent_and_custom_endpoints,
    CustomEndpoint,
    CustomEndpointsConfig,
};
```

**New:**
```rust
use helios_engine::{
    ServerBuilder,
    get, post, put, delete, patch,  // Helper functions
    EndpointBuilder,                 // For advanced cases
    EndpointResponse,                // For dynamic handlers
};
```

### Step 2: Convert Endpoint Definitions

#### Simple Static Endpoints

**Old:**
```rust
CustomEndpoint {
    method: "GET".to_string(),
    path: "/api/status".to_string(),
    response: serde_json::json!({"status": "ok"}),
    status_code: 200,
}
```

**New:**
```rust
get("/api/status", serde_json::json!({"status": "ok"}))
```

#### With Custom Status Codes

**Old:**
```rust
CustomEndpoint {
    method: "POST".to_string(),
    path: "/api/create".to_string(),
    response: serde_json::json!({"id": "123"}),
    status_code: 201,
}
```

**New:**
```rust
EndpointBuilder::post("/api/create")
    .handle(|_| {
        EndpointResponse::created(serde_json::json!({"id": "123"}))
    })
    .build()
```

### Step 3: Convert Server Startup

**Old:**
```rust
start_server_with_agent_and_custom_endpoints(
    agent,
    "model-name".to_string(),
    "127.0.0.1:8000",
    Some(custom_endpoints),
).await?;
```

**New:**
```rust
ServerBuilder::with_agent(agent, "model-name")
    .address("127.0.0.1:8000")
    .endpoints(endpoints)
    .serve()
    .await?;
```

## Complete Migration Example

### Before: Old API

```rust
use helios_engine::{
    Agent, Config, CalculatorTool,
    CustomEndpoint, CustomEndpointsConfig,
    start_server_with_agent_and_custom_endpoints,
};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let agent = Agent::builder("API Agent")
        .config(config)
        .tool(Box::new(CalculatorTool))
        .build()
        .await?;
    
    let custom_endpoints = CustomEndpointsConfig {
        endpoints: vec![
            CustomEndpoint {
                method: "GET".to_string(),
                path: "/api/version".to_string(),
                response: serde_json::json!({
                    "version": "0.4.5",
                    "service": "Helios Engine"
                }),
                status_code: 200,
            },
            CustomEndpoint {
                method: "GET".to_string(),
                path: "/api/status".to_string(),
                response: serde_json::json!({
                    "status": "operational"
                }),
                status_code: 200,
            },
            CustomEndpoint {
                method: "POST".to_string(),
                path: "/api/echo".to_string(),
                response: serde_json::json!({
                    "message": "Static response only - cannot echo your input"
                }),
                status_code: 200,
            },
        ],
    };
    
    start_server_with_agent_and_custom_endpoints(
        agent,
        "local-model".to_string(),
        "127.0.0.1:8000",
        Some(custom_endpoints),
    ).await?;
    
    Ok(())
}
```

### After: New API

```rust
use helios_engine::{
    Agent, Config, CalculatorTool,
    ServerBuilder, get, post,
    EndpointBuilder, EndpointResponse,
};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let agent = Agent::builder("API Agent")
        .config(config)
        .tool(Box::new(CalculatorTool))
        .build()
        .await?;
    
    let endpoints = vec![
        // Simple static endpoints
        get("/api/version", serde_json::json!({
            "version": "0.4.5",
            "service": "Helios Engine"
        })),
        
        get("/api/status", serde_json::json!({
            "status": "operational"
        })),
        
        // Dynamic endpoint that actually echoes your input!
        EndpointBuilder::post("/api/echo")
            .handle(|req| {
                let message = req
                    .and_then(|r| r.body)
                    .and_then(|b| b.get("message").cloned())
                    .unwrap_or(serde_json::json!("No message provided"));
                
                EndpointResponse::ok(serde_json::json!({
                    "echo": message,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            })
            .description("Echo your message back with timestamp")
            .build(),
    ];
    
    ServerBuilder::with_agent(agent, "local-model")
        .address("127.0.0.1:8000")
        .endpoints(endpoints)
        .serve()
        .await?;
    
    Ok(())
}
```

## New Features Not Available in Old API

### 1. Dynamic Request Handling

```rust
EndpointBuilder::post("/api/search")
    .handle(|req| {
        let req = req.unwrap();
        let query = req.query.get("q").cloned().unwrap_or_default();
        
        // Perform search based on query...
        
        EndpointResponse::ok(serde_json::json!({
            "results": [],
            "query": query
        }))
    })
    .build()
```

### 2. Request Body Access

```rust
EndpointBuilder::post("/api/users")
    .handle(|req| {
        let body = req.and_then(|r| r.body).unwrap_or_default();
        let name = body.get("name").and_then(|n| n.as_str());
        
        if let Some(name) = name {
            EndpointResponse::created(serde_json::json!({
                "id": "123",
                "name": name
            }))
        } else {
            EndpointResponse::bad_request("Name is required")
        }
    })
    .build()
```

### 3. Multiple Response Status Codes

```rust
EndpointBuilder::get("/api/users/:id")
    .handle(|req| {
        let user_exists = check_user_exists();
        
        if user_exists {
            EndpointResponse::ok(serde_json::json!({"id": "123"}))
        } else {
            EndpointResponse::not_found("User not found")
        }
    })
    .build()
```

### 4. Endpoint Descriptions

```rust
EndpointBuilder::get("/api/health")
    .json(serde_json::json!({"status": "healthy"}))
    .description("Health check endpoint")
    .build()
```

## Backward Compatibility

The old API is **still supported** for backward compatibility, so existing code will continue to work. However, we **strongly recommend** migrating to the new API for:

- Better developer experience
- More functionality
- Future-proof code
- Consistency with the rest of the framework

## FAQs

### Q: Do I need to migrate immediately?
**A:** No, the old API still works. But we recommend migrating to get the benefits of the new API.

### Q: Can I mix old and new APIs?
**A:** No, use one or the other. The new API is recommended.

### Q: What happens to `load_custom_endpoints_config()`?
**A:** It still works for loading TOML config files with the old format. For new projects, we recommend defining endpoints in code using the new API.

### Q: How do I add many endpoints?
**A:** Use a vector! Just like how you add multiple agents:
```rust
let endpoints = vec![
    get("/api/v1", serde_json::json!({"version": "1"})),
    get("/api/v2", serde_json::json!({"version": "2"})),
    // ... more endpoints
];

ServerBuilder::with_agent(agent, "model")
    .endpoints(endpoints)
    .serve()
    .await?;
```

## Need Help?

- See [CUSTOM_ENDPOINTS.md](CUSTOM_ENDPOINTS.md) for complete documentation
- Check [examples/serve_simple_endpoints.rs](../examples/serve_simple_endpoints.rs) for a working example
- Compare with [examples/serve_with_custom_endpoints.rs](../examples/serve_with_custom_endpoints.rs) (old API) to see the differences

## Summary of Changes

| Feature | Old API | New API |
|---------|---------|---------|
| Endpoint creation | Manual struct | Helper functions + Builder |
| Static responses |  |  |
| Dynamic responses | ❌ |  |
| Request data access | ❌ |  |
| Status code helpers | ❌ |  |
| Builder pattern | ❌ |  |
| Descriptions | ❌ |  |
| Vector support |  |  |
| Code reduction | - | ~70% less code |

The new API reduces code by approximately **70%** while adding more functionality! 🎉
