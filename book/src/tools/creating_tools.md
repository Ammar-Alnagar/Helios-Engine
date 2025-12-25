# Creating Custom Tools

Helios Engine provides a flexible system for creating custom tools. This chapter will cover the two main ways to create custom tools: using the `ToolBuilder` (the easy way) and implementing the `Tool` trait directly (the advanced way).

## Using `ToolBuilder` (Recommended)

The `ToolBuilder` provides a simplified way to create custom tools without implementing the `Tool` trait manually. This is the recommended approach for most use cases.

### `quick_tool!` Macro

The easiest way to create a tool is with the `quick_tool!` macro. It handles all the boilerplate for you, including parameter extraction and type conversion.

```rust
use helios_engine::quick_tool;

// Create a tool in ONE expression!
let volume_tool = quick_tool! {
    name: calculate_volume,
    description: "Calculate the volume of a box",
    params: (width: f64, height: f64, depth: f64),
    execute: |width, height, depth| {
        format!("Volume: {:.2} cubic meters", width * height * depth)
    }
};
```

### `ToolBuilder` API

If you need more control, you can use the `ToolBuilder` API directly.

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

## Implementing the `Tool` Trait (Advanced)

For advanced use cases or when you need more control, you can implement the `Tool` trait directly.

```rust
use async_trait::async_trait;
use helios_engine::{Tool, ToolParameter, ToolResult};
use serde_json::Value;
use std::collections::HashMap;

struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "get_weather"
    }

    fn description(&self) -> &str {
        "Get the current weather for a location"
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "location".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "City name or location".to_string(),
                required: Some(true),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> helios_engine::Result<ToolResult> {
        let location = args["location"]
            .as_str()
            .ok_or_else(|| helios_engine::HeliosError::ToolError("location is required".to_string()))?;

        // Your weather API logic here
        let weather_data = format!("The weather in {} is sunny, 72°F", location);

        Ok(ToolResult::success(weather_data))
    }
}
```
