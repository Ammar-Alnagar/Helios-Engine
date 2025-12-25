# Tool Builder

The `ToolBuilder` provides a simplified way to create custom tools without implementing the `Tool` trait manually. This is the recommended approach for most use cases.

## `quick_tool!` Macro

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

## `ToolBuilder` API

If you need more control, you can use the `ToolBuilder` API directly.

### Creating a `ToolBuilder`

You can create a new `ToolBuilder` using the `ToolBuilder::new()` method.

```rust
use helios_engine::ToolBuilder;

let tool_builder = ToolBuilder::new("my_tool");
```

### Configuring a `ToolBuilder`

The `ToolBuilder` provides several methods for configuring a tool:

- **`description(description: impl Into<String>)`**: Sets the description of the tool.
- **`parameter(name: impl Into<String>, param_type: impl Into<String>, description: impl Into<String>, required: bool)`**: Adds a parameter to the tool.
- **`optional_parameter(name: impl Into<String>, param_type: impl Into<String>, description: impl Into<String>)`**: Adds an optional parameter to the tool.
- **`required_parameter(name: impl Into<String>, param_type: impl Into<String>, description: impl Into<String>)`**: Adds a required parameter to the tool.
- **`parameters(params: impl Into<String>)`**: Adds multiple parameters at once using a compact format.
- **`function<F, Fut>(f: F)`**: Sets the function to execute when the tool is called.
- **`sync_function<F>(f: F)`**: Sets the function using a synchronous closure.
- **`ftool<F, T1, T2, R>(f: F)`**: Ultra-simple API: Pass a function directly with automatic type inference.
- **`ftool3<F, T1, T2, T3, R>(f: F)`**: Ultra-simple API: Pass a 3-parameter function directly with automatic type inference.
- **`ftool4<F, T1, T2, T3, T4, R>(f: F)`**: Ultra-simple API: Pass a 4-parameter function directly with automatic type inference.

### Building a Tool

Once you've configured your tool, you can build it using the `build()` method.

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

You can also use the `try_build()` method, which returns a `Result` instead of panicking if the tool is not configured correctly.
