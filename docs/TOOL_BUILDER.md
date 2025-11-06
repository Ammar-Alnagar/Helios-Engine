# Tool Builder Guide

## Overview

The `ToolBuilder` provides a simplified way to create custom tools for Helios Engine agents without manually implementing the `Tool` trait. This builder pattern allows you to quickly wrap existing functions as tools that agents can use.

## Why Use ToolBuilder?

**Before (Manual Implementation):**
```rust
use async_trait::async_trait;
use helios_engine::{Tool, ToolParameter, ToolResult};
use serde_json::Value;
use std::collections::HashMap;

struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str {
        "my_tool"
    }

    fn description(&self) -> &str {
        "Does something useful"
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "input".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "The input value".to_string(),
                required: Some(true),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> helios_engine::Result<ToolResult> {
        let input = args
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| helios_engine::HeliosError::ToolError(
                "Missing input parameter".to_string()
            ))?;
        
        Ok(ToolResult::success(format!("Processed: {}", input)))
    }
}
```

**After (Using ToolBuilder):**
```rust
use helios_engine::{ToolBuilder, ToolResult};
use serde_json::Value;

let tool = ToolBuilder::new("my_tool")
    .description("Does something useful")
    .required_parameter("input", "string", "The input value")
    .sync_function(|args: Value| {
        let input = args.get("input").and_then(|v| v.as_str())
            .ok_or_else(|| helios_engine::HeliosError::ToolError(
                "Missing input parameter".to_string()
            ))?;
        
        Ok(ToolResult::success(format!("Processed: {}", input)))
    })
    .build();
```

## Basic Usage

### Creating a Simple Tool

```rust
use helios_engine::{ToolBuilder, ToolResult};
use serde_json::Value;

// Define your function logic
async fn add_numbers(args: Value) -> helios_engine::Result<ToolResult> {
    let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
    Ok(ToolResult::success((a + b).to_string()))
}

// Build the tool
let tool = ToolBuilder::new("add_numbers")
    .description("Add two numbers together")
    .required_parameter("a", "number", "First number")
    .required_parameter("b", "number", "Second number")
    .function(add_numbers)
    .build();
```

### Using with an Agent

```rust
use helios_engine::{Agent, Config, ToolBuilder, ToolResult};
use serde_json::Value;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    // Create a tool
    let calculator = ToolBuilder::new("multiply")
        .description("Multiply two numbers")
        .required_parameter("x", "number", "First number")
        .required_parameter("y", "number", "Second number")
        .sync_function(|args: Value| {
            let x = args.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let y = args.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
            Ok(ToolResult::success((x * y).to_string()))
        })
        .build();
    
    // Create an agent with the tool
    let mut agent = Agent::builder("MathAgent")
        .config(config)
        .tool(calculator)
        .build()
        .await?;
    
    let response = agent.chat("What is 7 times 8?").await?;
    println!("Agent: {}", response);
    
    Ok(())
}
```

## API Reference

### ToolBuilder Methods

#### `new(name: impl Into<String>) -> Self`

Creates a new `ToolBuilder` with the given name.

```rust
let builder = ToolBuilder::new("my_tool");
```

#### `description(description: impl Into<String>) -> Self`

Sets the description of the tool. This helps the LLM understand when to use the tool.

```rust
let builder = ToolBuilder::new("weather")
    .description("Get current weather information for a location");
```

#### `parameter(name, param_type, description, required) -> Self`

Adds a parameter to the tool.

**Parameter Types:**
- `"string"` - Text values
- `"number"` - Numeric values (integers or floats)
- `"boolean"` - True/false values
- `"object"` - JSON objects
- `"array"` - JSON arrays

```rust
let builder = ToolBuilder::new("search")
    .parameter("query", "string", "Search query", true)
    .parameter("limit", "number", "Maximum results", false);
```

#### `required_parameter(name, param_type, description) -> Self`

Convenience method to add a required parameter.

#### `optional_parameter(name, param_type, description) -> Self`

Convenience method to add an optional parameter.

#### `function<F, Fut>(f: F) -> Self`

Sets an async function to execute when the tool is called.

#### `sync_function<F>(f: F) -> Self`

Sets a synchronous function to execute when the tool is called.

#### `build() -> Box<dyn Tool>`

Builds the tool, consuming the builder. Panics if the function has not been set.

#### `try_build() -> Result<Box<dyn Tool>>`

Builds the tool, returning a `Result` instead of panicking.

## Advanced Patterns

### Wrapping Existing Functions

```rust
// Your existing function
fn calculate_discount(price: f64, discount_percent: f64) -> f64 {
    price * (1.0 - discount_percent / 100.0)
}

// Wrap it as a tool
let discount_tool = ToolBuilder::new("calculate_discount")
    .description("Calculate discounted price")
    .required_parameter("price", "number", "Original price")
    .required_parameter("discount_percent", "number", "Discount percentage")
    .sync_function(|args: Value| {
        let price = args.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let discount = args.get("discount_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
        
        let result = calculate_discount(price, discount);
        Ok(ToolResult::success(format!("${:.2}", result)))
    })
    .build();
```

### Closures with Captured Variables

```rust
let multiplier = 10;

let tool = ToolBuilder::new("multiply_by_constant")
    .description("Multiply a number by a fixed value")
    .required_parameter("value", "number", "Value to multiply")
    .sync_function(move |args: Value| {
        let value = args.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
        Ok(ToolResult::success((value * multiplier as f64).to_string()))
    })
    .build();
```

### Error Handling

```rust
let validator_tool = ToolBuilder::new("validate_email")
    .description("Validate an email address")
    .required_parameter("email", "string", "Email to validate")
    .sync_function(|args: Value| {
        let email = args.get("email")
            .and_then(|v| v.as_str())
            .ok_or_else(|| helios_engine::HeliosError::ToolError(
                "Missing email parameter".to_string()
            ))?;
        
        if email.contains('@') && email.contains('.') {
            Ok(ToolResult::success(format!("{} is valid", email)))
        } else {
            Ok(ToolResult::error(format!("{} is not a valid email", email)))
        }
    })
    .build();
```

## Complete Example

See `examples/tool_builder_demo.rs` for a comprehensive example demonstrating:
- Wrapping existing functions
- Async operations
- Optional parameters
- Closure capture
- Multiple tools in one agent

## Best Practices

1. **Clear Descriptions**: Write clear descriptions for tools and parameters to help the LLM choose the right tool
2. **Parameter Validation**: Always validate required parameters and provide helpful error messages
3. **Type Safety**: Use appropriate parameter types (`string`, `number`, `boolean`, etc.)
4. **Error Handling**: Handle errors gracefully using `Result` and `ToolResult`
5. **Async When Needed**: Use `function()` for async operations (API calls, I/O), `sync_function()` for simple computations

## See Also

- [Tools Guide](TOOLS.md) - Complete tool system documentation
- [API Reference](API.md) - Full API documentation
- [Examples](../examples/) - More example implementations
