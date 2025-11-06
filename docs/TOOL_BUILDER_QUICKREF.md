# Tool Builder Quick Reference

## Quick Start

```rust
use helios_engine::{ToolBuilder, ToolResult};
use serde_json::Value;

// Create a simple tool
let tool = ToolBuilder::new("tool_name")
    .description("What the tool does")
    .required_parameter("param1", "string", "Description")
    .sync_function(|args: Value| {
        // Your logic here
        Ok(ToolResult::success("result"))
    })
    .build();

// Use with agent
let mut agent = Agent::builder("MyAgent")
    .config(config)
    .tool(tool)
    .build()
    .await?;
```

## Common Patterns

### Wrap Existing Synchronous Function

```rust
fn my_calculation(x: f64, y: f64) -> f64 {
    x * y + 10.0
}

let tool = ToolBuilder::new("calculate")
    .description("Perform calculation")
    .required_parameter("x", "number", "First value")
    .required_parameter("y", "number", "Second value")
    .sync_function(|args: Value| {
        let x = args.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let y = args.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let result = my_calculation(x, y);
        Ok(ToolResult::success(result.to_string()))
    })
    .build();
```

### Wrap Async Function

```rust
async fn fetch_data(id: &str) -> Result<String, String> {
    // Async operation
    Ok(format!("Data for {}", id))
}

let tool = ToolBuilder::new("fetch")
    .description("Fetch data by ID")
    .required_parameter("id", "string", "Resource ID")
    .function(|args: Value| async move {
        let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("");
        match fetch_data(id).await {
            Ok(data) => Ok(ToolResult::success(data)),
            Err(e) => Ok(ToolResult::error(e)),
        }
    })
    .build();
```

### Optional Parameters

```rust
let tool = ToolBuilder::new("format_text")
    .description("Format text with optional settings")
    .required_parameter("text", "string", "Text to format")
    .optional_parameter("uppercase", "boolean", "Convert to uppercase")
    .optional_parameter("prefix", "string", "Prefix to add")
    .sync_function(|args: Value| {
        let mut text = args.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        if args.get("uppercase").and_then(|v| v.as_bool()).unwrap_or(false) {
            text = text.to_uppercase();
        }
        
        if let Some(prefix) = args.get("prefix").and_then(|v| v.as_str()) {
            text = format!("{}{}", prefix, text);
        }
        
        Ok(ToolResult::success(text))
    })
    .build();
```

### Capture External State

```rust
let api_key = "secret_key".to_string();
let base_url = "https://api.example.com";

let tool = ToolBuilder::new("api_call")
    .description("Call external API")
    .required_parameter("endpoint", "string", "API endpoint")
    .function(move |args: Value| {
        let key = api_key.clone();
        let url = base_url.to_string();
        
        async move {
            let endpoint = args.get("endpoint").and_then(|v| v.as_str()).unwrap_or("");
            let full_url = format!("{}/{}", url, endpoint);
            // Use key and full_url in API call
            Ok(ToolResult::success(format!("Called: {}", full_url)))
        }
    })
    .build();
```

### Error Handling

```rust
let tool = ToolBuilder::new("validate")
    .description("Validate input")
    .required_parameter("email", "string", "Email to validate")
    .sync_function(|args: Value| {
        let email = args.get("email")
            .and_then(|v| v.as_str())
            .ok_or_else(|| helios_engine::HeliosError::ToolError(
                "Missing email parameter".to_string()
            ))?;
        
        if email.contains('@') {
            Ok(ToolResult::success(format!("{} is valid", email)))
        } else {
            Ok(ToolResult::error(format!("{} is invalid", email)))
        }
    })
    .build();
```

### Complex JSON Parameters

```rust
let tool = ToolBuilder::new("process_order")
    .description("Process an order")
    .required_parameter("order", "object", "Order details")
    .sync_function(|args: Value| {
        let order = args.get("order").ok_or_else(|| {
            helios_engine::HeliosError::ToolError("Missing order".to_string())
        })?;
        
        let customer = order.get("customer").and_then(|v| v.as_str());
        let total = order.get("total").and_then(|v| v.as_f64());
        
        Ok(ToolResult::success(format!(
            "Order for {} - ${:.2}",
            customer.unwrap_or("unknown"),
            total.unwrap_or(0.0)
        )))
    })
    .build();
```

## Parameter Types

| Type | JSON Type | Rust Extraction |
|------|-----------|----------------|
| `"string"` | String | `.as_str()` |
| `"number"` | Number | `.as_f64()` or `.as_i64()` |
| `"boolean"` | Boolean | `.as_bool()` |
| `"object"` | Object | `.as_object()` |
| `"array"` | Array | `.as_array()` |

## Builder Methods

```rust
.new(name)                          // Create builder
.description(desc)                  // Set description
.parameter(name, type, desc, req)   // Add parameter
.required_parameter(name, type, desc) // Required param
.optional_parameter(name, type, desc) // Optional param
.function(async_fn)                 // Async function
.sync_function(sync_fn)             // Sync function
.build()                            // Build (panics)
.try_build()                        // Build (Result)
```

## Common Mistakes

### ❌ Forgot to set function
```rust
// Will panic!
let tool = ToolBuilder::new("tool")
    .description("desc")
    .build(); // ERROR: No function set
```

### ✅ Use try_build for safety
```rust
let tool = ToolBuilder::new("tool")
    .description("desc")
    .sync_function(|_| Ok(ToolResult::success("ok")))
    .try_build()?; // Returns Result
```

### ❌ Moving captured variables
```rust
let data = String::from("test");
let tool = ToolBuilder::new("tool")
    .function(|_| async {
        println!("{}", data); // ERROR: data moved
        Ok(ToolResult::success("ok"))
    })
    .build();
```

### ✅ Clone or use move
```rust
let data = String::from("test");
let tool = ToolBuilder::new("tool")
    .function(move |_| async move {
        println!("{}", data); // OK: moved into closure
        Ok(ToolResult::success("ok"))
    })
    .build();
```

## See Also

- [Complete Guide](TOOL_BUILDER.md) - Full documentation
- [Examples](../examples/tool_builder_demo.rs) - Working examples
- [API Reference](API.md) - API documentation
