# Tools Guide

Helios Engine includes 16+ built-in tools for common tasks, and provides a flexible system for creating custom tools. Tools allow agents to perform actions beyond just text generation, enabling them to interact with files, execute commands, access web resources, and manipulate data.

## Overview

Tools in Helios Engine follow a simple pattern:
1. **Definition**: Each tool defines its name, description, and parameters
2. **Execution**: Tools receive JSON parameters and return structured results
3. **Registration**: Tools are registered with agents during creation

## Built-in Tools

### Core Tools

#### CalculatorTool
Performs mathematical calculations and evaluations.

```rust
use helios_engine::CalculatorTool;

let mut agent = Agent::builder("MathAgent")
    .config(config)
    .tool(Box::new(CalculatorTool))
    .build()
    .await?;
```

**Parameters:**
- `expression` (string, required): Mathematical expression to evaluate

**Example Usage:**
```rust
let result = agent.chat("Calculate 15 * 7 + 3").await?;
```

#### EchoTool
Simply echoes back the input message (useful for testing).

```rust
use helios_engine::EchoTool;

agent.tool(Box::new(EchoTool));
```

**Parameters:**
- `message` (string, required): Message to echo back

### File Management Tools

#### FileSearchTool
Search for files by name pattern or content within files.

```rust
use helios_engine::FileSearchTool;

agent.tool(Box::new(FileSearchTool));
```

**Parameters:**
- `path` (string, optional): Directory path to search (default: current directory)
- `pattern` (string, optional): File name pattern with wildcards (e.g., `*.rs`)
- `content` (string, optional): Text content to search for within files
- `max_results` (number, optional): Maximum number of results (default: 50)

**Examples:**
```rust
// Find all Rust files
agent.chat("Find all .rs files").await?;

// Search for specific content
agent.chat("Find files containing 'TODO'").await?;
```

#### FileReadTool
Read the contents of a file with optional line range selection.

```rust
use helios_engine::FileReadTool;

agent.tool(Box::new(FileReadTool));
```

**Parameters:**
- `path` (string, required): File path to read
- `start_line` (number, optional): Starting line number (1-indexed)
- `end_line` (number, optional): Ending line number (1-indexed)

**Examples:**
```rust
// Read entire file
agent.chat("Read the file config.toml").await?;

// Read specific lines
agent.chat("Read lines 10-20 of main.rs").await?;
```

#### FileWriteTool
Write content to a file (creates new or overwrites existing).

```rust
use helios_engine::FileWriteTool;

agent.tool(Box::new(FileWriteTool));
```

**Parameters:**
- `path` (string, required): File path to write to
- `content` (string, required): Content to write

**Example:**
```rust
agent.chat("Create a new file called notes.txt with content 'Hello World'").await?;
```

#### FileEditTool
Edit a file by replacing specific text (find and replace).

```rust
use helios_engine::FileEditTool;

agent.tool(Box::new(FileEditTool));
```

**Parameters:**
- `path` (string, required): File path to edit
- `find` (string, required): Text to find
- `replace` (string, required): Replacement text

**Example:**
```rust
agent.chat("In main.rs, replace 'old_function' with 'new_function'").await?;
```

#### FileIOTool
Unified file operations: read, write, append, delete, copy, move, exists, size.

```rust
use helios_engine::FileIOTool;

agent.tool(Box::new(FileIOTool));
```

**Parameters:**
- `operation` (string, required): Operation type (read, write, append, delete, copy, move, exists)
- `path` (string, required): File path
- Additional parameters depending on operation

#### FileListTool
List directory contents with detailed metadata.

```rust
use helios_engine::FileListTool;

agent.tool(Box::new(FileListTool));
```

**Parameters:**
- `path` (string, optional): Directory path to list
- `show_hidden` (boolean, optional): Show hidden files
- `recursive` (boolean, optional): List recursively
- `max_depth` (number, optional): Maximum recursion depth

### Web & API Tools

#### WebScraperTool
Fetch and extract content from web URLs.

```rust
use helios_engine::WebScraperTool;

agent.tool(Box::new(WebScraperTool));
```

**Parameters:**
- `url` (string, required): URL to scrape
- `extract_text` (boolean, optional): Extract readable text from HTML
- `timeout_seconds` (number, optional): Request timeout

#### HttpRequestTool
Make HTTP requests with various methods.

```rust
use helios_engine::HttpRequestTool;

agent.tool(Box::new(HttpRequestTool));
```

**Parameters:**
- `method` (string, required): HTTP method (GET, POST, PUT, DELETE, etc.)
- `url` (string, required): Request URL
- `headers` (object, optional): Request headers
- `body` (string, optional): Request body
- `timeout_seconds` (number, optional): Request timeout

#### JsonParserTool
Parse, validate, format, and manipulate JSON data.

```rust
use helios_engine::JsonParserTool;

agent.tool(Box::new(JsonParserTool));
```

**Operations:**
- `parse` - Parse and validate JSON
- `stringify` - Format JSON with optional indentation
- `get_value` - Extract values by JSON path
- `set_value` - Modify JSON values
- `validate` - Check JSON validity

### System & Utility Tools

#### ShellCommandTool
Execute shell commands safely with security restrictions.

```rust
use helios_engine::ShellCommandTool;

agent.tool(Box::new(ShellCommandTool));
```

**Parameters:**
- `command` (string, required): Shell command to execute
- `timeout_seconds` (number, optional): Command timeout

#### SystemInfoTool
Retrieve system information (OS, CPU, memory, disk, network).

```rust
use helios_engine::SystemInfoTool;

agent.tool(Box::new(SystemInfoTool));
```

**Parameters:**
- `category` (string, optional): Info category (all, os, cpu, memory, disk, network)

#### TimestampTool
Work with timestamps and date/time operations.

```rust
use helios_engine::TimestampTool;

agent.tool(Box::new(TimestampTool));
```

**Operations:**
- `now` - Current time
- `format` - Format timestamps
- `parse` - Parse timestamp strings
- `add`/`subtract` - Time arithmetic
- `diff` - Time difference calculation

#### TextProcessorTool
Process and manipulate text with various operations.

```rust
use helios_engine::TextProcessorTool;

agent.tool(Box::new(TextProcessorTool));
```

**Operations:**
- `search` - Regex-based text search
- `replace` - Find and replace with regex
- `split`/`join` - Text splitting and joining
- `count` - Character, word, and line counts
- `uppercase`/`lowercase` - Case conversion
- `trim` - Whitespace removal
- `lines`/`words` - Text formatting

### Data Storage Tools

#### MemoryDBTool
In-memory key-value database for caching data during conversations.

```rust
use helios_engine::MemoryDBTool;

agent.tool(Box::new(MemoryDBTool::new()));
```

**Operations:**
- `set` - Store key-value pairs
- `get` - Retrieve values
- `delete` - Remove entries
- `list` - Show all stored data
- `clear` - Remove all data
- `exists` - Check key existence

#### QdrantRAGTool
RAG (Retrieval-Augmented Generation) tool with Qdrant vector database.

```rust
use helios_engine::QdrantRAGTool;

let rag_tool = QdrantRAGTool::new(
    "http://localhost:6333",                    // Qdrant URL
    "my_collection",                             // Collection name
    "https://api.openai.com/v1/embeddings",     // Embedding API
    std::env::var("OPENAI_API_KEY").unwrap(),   // API key
);

agent.tool(Box::new(rag_tool));
```

**Operations:**
- `add_document` - Store and embed documents
- `search` - Semantic search
- `delete` - Remove documents
- `clear` - Clear collection

## Creating Custom Tools

### 🆕 Easy Way: Using ToolBuilder (Recommended)

The **ToolBuilder** provides a simplified way to create custom tools without implementing the Tool trait manually. This is the recommended approach for most use cases.

> **✨ NEW: `quick_tool!` Macro - The EASIEST Way!**  
> We've added the `quick_tool!` macro that makes tool creation incredibly simple with ZERO boilerplate.  
> See the [Quick Start](#quick-start-quick_tool-macro) below or the full [Simplified Tool Builder Guide](TOOL_BUILDER_SIMPLIFIED.md).

#### Why Use ToolBuilder?

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

#### Quick Start

```rust
use helios_engine::{Agent, Config, ToolBuilder, ToolResult};
use serde_json::Value;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    // Create a tool in just a few lines!
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
    
    // Use it with an agent
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

#### ToolBuilder API Reference

**Builder Methods:**

- `new(name)` - Create a new ToolBuilder with the given name
- `description(desc)` - Set the tool description
- `parameter(name, type, desc, required)` - Add a parameter
- `required_parameter(name, type, desc)` - Add a required parameter
- `optional_parameter(name, type, desc)` - Add an optional parameter
- `function(async_fn)` - Set an async function to execute
- `sync_function(sync_fn)` - Set a synchronous function to execute
- `build()` - Build the tool (panics if function not set)
- `try_build()` - Build the tool (returns Result)

**Parameter Types:**
- `"string"` - Text values
- `"number"` - Numeric values (integers or floats)
- `"boolean"` - True/false values
- `"object"` - JSON objects
- `"array"` - JSON arrays

#### Quick Start: `quick_tool!` Macro

⭐ **This is the EASIEST way to create tools!** Zero boilerplate, automatic parameter extraction:

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

// Another example - BMI calculator
let bmi_tool = quick_tool! {
    name: calculate_bmi,
    description: "Calculate Body Mass Index",
    params: (weight_kg: f64, height_m: f64),
    execute: |weight, height| {
        let bmi = weight / (height * height);
        format!("BMI: {:.1}", bmi)
    }
};

// Works with different types too!
let greet_tool = quick_tool! {
    name: greet_user,
    description: "Greet a user",
    params: (name: String, formal: bool),
    execute: |name, formal| {
        if formal {
            format!("Good day, {}.", name)
        } else {
            format!("Hey {}!", name)
        }
    }
};
```

**Supported types**: `i32`, `i64`, `u32`, `u64`, `f32`, `f64`, `bool`, `String`

**What it does automatically:**
- Extracts parameters from JSON
- Handles type conversion
- Provides sensible defaults
- Zero manual parameter handling!

For complete documentation and alternative methods, see [TOOL_BUILDER_SIMPLIFIED.md](TOOL_BUILDER_SIMPLIFIED.md).

#### ToolBuilder Patterns

**Wrapping Existing Functions:**

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

**Async Operations:**

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

**Optional Parameters:**

```rust
let tool = ToolBuilder::new("greet")
    .description("Greet someone")
    .required_parameter("name", "string", "Name of person")
    .optional_parameter("title", "string", "Optional title")
    .sync_function(|args: Value| {
        let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("stranger");
        let title = args.get("title").and_then(|v| v.as_str());
        
        let greeting = if let Some(t) = title {
            format!("Hello, {} {}!", t, name)
        } else {
            format!("Hello, {}!", name)
        };
        
        Ok(ToolResult::success(greeting))
    })
    .build();
```

**Capturing External State:**

```rust
let api_key = "secret_key".to_string();
let multiplier = 10;

let tool = ToolBuilder::new("api_multiply")
    .description("Multiply a number and use captured state")
    .required_parameter("value", "number", "Value to multiply")
    .function(move |args: Value| {
        let key = api_key.clone();
        let mult = multiplier;
        
        async move {
            let value = args.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let result = value * mult as f64;
            // Use key in API call...
            Ok(ToolResult::success(result.to_string()))
        }
    })
    .build();
```

**Error Handling:**

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

**Complex JSON Parameters:**

```rust
let tool = ToolBuilder::new("process_order")
    .description("Process a customer order")
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

#### ToolBuilder Best Practices

1. **Clear Descriptions**: Write clear descriptions for tools and parameters to help the LLM choose the right tool
2. **Parameter Validation**: Always validate required parameters and provide helpful error messages
3. **Type Safety**: Use appropriate parameter types (`string`, `number`, `boolean`, etc.)
4. **Error Handling**: Handle errors gracefully using `Result` and `ToolResult`
5. **Async When Needed**: Use `function()` for async operations (API calls, I/O), `sync_function()` for simple computations

#### Complete Example

See `examples/tool_builder_demo.rs` for a comprehensive example demonstrating:
- Wrapping existing functions
- Async operations
- Optional parameters
- Closure capture
- Multiple tools in one agent

---

### Advanced Way: Implementing the Tool Trait

For advanced use cases or when you need more control, you can implement the `Tool` trait directly. Tools must be thread-safe and handle errors gracefully.

#### Basic Tool Structure

### Advanced Way: Implementing the Tool Trait

For advanced use cases or when you need more control, you can implement the `Tool` trait directly. Tools must be thread-safe and handle errors gracefully.

#### Basic Tool Structure

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
            .ok_or_else(|| helios_engine::Error::InvalidParameter("location".to_string()))?;

        // Your weather API logic here
        let weather_data = fetch_weather_data(location).await?;

        Ok(ToolResult::success(format!(
            "Weather in {}: {}°, {}",
            location, weather_data.temperature, weather_data.condition
        )))
    }
}
```

### Complete Example

```rust
use async_trait::async_trait;
use helios_engine::{Tool, ToolParameter, ToolResult, Agent, Config};
use serde_json::Value;
use std::collections::HashMap;

struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "get_weather"
    }

    fn description(&self) -> &str {
        "Get current weather information for a location"
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "location".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "City name (e.g., 'New York', 'London, UK')".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "unit".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Temperature unit: 'celsius' or 'fahrenheit'".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> helios_engine::Result<ToolResult> {
        let location = args["location"]
            .as_str()
            .ok_or_else(|| helios_engine::Error::InvalidParameter("location is required".to_string()))?;

        let unit = args["unit"]
            .as_str()
            .unwrap_or("celsius");

        // Simulate weather API call
        let temperature = 22;
        let condition = "Sunny";

        let temp_display = match unit {
            "fahrenheit" => format!("{}°F", temperature * 9/5 + 32),
            _ => format!("{}°C", temperature),
        };

        Ok(ToolResult::success(format!(
            "Weather in {}: {}, {}",
            location, temp_display, condition
        )))
    }
}

// Use your custom tool
#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    let mut agent = Agent::builder("WeatherAgent")
        .config(config)
        .system_prompt("You are a helpful assistant with access to weather information.")
        .tool(Box::new(WeatherTool))
        .build()
        .await?;

    let response = agent.chat("What's the weather like in Tokyo?").await?;
    println!("{}", response);

    Ok(())
}
```

## Tool Best Practices

### Error Handling
- Always handle errors gracefully in your `execute` method
- Return appropriate `ToolResult` types for different outcomes
- Provide meaningful error messages

### Parameter Validation
- Validate required parameters early
- Provide sensible defaults for optional parameters
- Use clear parameter names and descriptions

### Performance
- Keep tool execution reasonably fast
- Avoid blocking operations when possible
- Consider implementing timeouts for external API calls

### Security
- Validate file paths to prevent directory traversal
- Sanitize command inputs for shell tools
- Be cautious with network requests and API keys

### Naming Conventions
- Use lowercase with underscores for tool names: `file_search`, `web_scraper`
- Make descriptions clear and actionable
- Parameter names should be descriptive but concise

## Advanced Tool Patterns

### Stateful Tools
Tools can maintain state between executions:

```rust
use std::sync::Mutex;

struct CounterTool {
    count: Mutex<i32>,
}

#[async_trait]
impl Tool for CounterTool {
    fn name(&self) -> &str {
        "counter"
    }

    fn description(&self) -> &str {
        "A simple counter that maintains state"
    }

    // ... parameters and execute methods
}
```

### Async Tools
Tools can perform async operations naturally since `execute` is async:

```rust
async fn execute(&self, args: Value) -> helios_engine::Result<ToolResult> {
    // Perform async HTTP request
    let response = reqwest::get("https://api.example.com/data").await?;
    let data: serde_json::Value = response.json().await?;

    Ok(ToolResult::success(format!("Fetched: {}", data)))
}
```

### Tool Composition
Create complex tools by combining simpler ones:

```rust
struct DataProcessorTool {
    file_tool: FileReadTool,
    json_tool: JsonParserTool,
}

#[async_trait]
impl Tool for DataProcessorTool {
    // Implementation that uses both tools internally
}
```

## Tool Registry

The `ToolRegistry` manages all available tools:

```rust
use helios_engine::ToolRegistry;

// Create registry
let mut registry = ToolRegistry::new();

// Register tools
registry.register(Box::new(CalculatorTool));
registry.register(Box::new(FileReadTool));

// List available tools
let tool_names = registry.list_tools();
println!("Available tools: {:?}", tool_names);

// Execute tools directly
let result = registry.execute("calculator", serde_json::json!({
    "expression": "2 + 2"
})).await?;
```

## Next Steps

- **[Examples](../examples/)** - See tools in action
- **[API Reference](API.md)** - Complete Tool trait documentation
- **[Usage Guide](USAGE.md)** - More usage patterns
