# Tool Creation Guide

This guide shows you the **simplest way** to create tools in Helios Engine using the `quick_tool!` macro.

## Table of Contents

1. [Overview](#overview)
2. [The `quick_tool!` Macro](#the-quick_tool-macro) ⭐ **Recommended**
3. [Alternative Methods](#alternative-methods)
4. [Quick Examples](#quick-examples)
5. [Advanced Usage](#advanced-usage)
6. [Migration Guide](#migration-guide)

## Overview

Creating tools in Helios Engine is incredibly easy. The `quick_tool!` macro does all the heavy lifting:
- Automatically extracts parameters from JSON
- Maps Rust types to JSON schema types  
- Handles all boilerplate code
- One simple expression to create a complete tool

## The `quick_tool!` Macro

⭐ **This is the recommended and easiest way to create tools!**

### The Philosophy

You already have functions. We just need to wrap them. That's it!

### Quick Start

**Step 1: Write Your Function**

```rust
fn adder(x: i32, y: i32) -> i32 {
    x + y
}
```

**Step 2: Wrap It**

```rust
use helios_engine::quick_tool;

let tool = quick_tool! {
    name: adder,
    description: "Add two numbers together",
    params: (x: i32, y: i32),
    execute: |x, y| {
        format!("Result: {}", adder(x, y))
    }
};
```

**Step 3: Use It**

```rust
let mut agent = Agent::builder("MyAgent")
    .config(config)
    .tool(tool)
    .build()
    .await?;
```

That's it! No boilerplate, no complex trait implementations, no manual JSON parsing.

### Macro Syntax

```rust
quick_tool! {
    name: function_name,           // The name of the tool
    description: "what it does",   // Human-readable description
    params: (param1: type1, param2: type2),  // Parameter list with types
    execute: |param1, param2| {    // The actual logic
        // Your code here
    }
}
```

### What It Does Automatically

✅ Extracts parameters from JSON  
✅ Converts JSON types to Rust types  
✅ Provides default values for missing parameters  
✅ Formats the output as a string  
✅ Handles all the Tool trait implementation  

You just write the logic!

### Supported Types

The macro supports these types out of the box:

| Rust Type | JSON Type | Default Value |
|-----------|-----------|---------------|
| `i32`, `i64`, `u32`, `u64` | integer | `0` |
| `f32`, `f64` | number | `0.0` |
| `bool` | boolean | `false` |
| `String` | string | `""` |

## Quick Examples

### Example 1: Calculator Functions

```rust
// Your normal functions
fn add(a: i32, b: i32) -> i32 { a + b }
fn subtract(a: i32, b: i32) -> i32 { a - b }
fn multiply(a: i32, b: i32) -> i32 { a * b }
fn divide(a: f64, b: f64) -> f64 { a / b }

// Wrap them
let add_tool = quick_tool! {
    name: add,
    description: "Add two integers",
    params: (a: i32, b: i32),
    execute: |a, b| format!("{} + {} = {}", a, b, add(a, b))
};

let divide_tool = quick_tool! {
    name: divide,
    description: "Divide two numbers",
    params: (a: f64, b: f64),
    execute: |a, b| {
        if b == 0.0 {
            "Cannot divide by zero".to_string()
        } else {
            format!("{} ÷ {} = {:.2}", a, b, divide(a, b))
        }
    }
};
```

### Example 2: String Processing

```rust
fn capitalize(text: String) -> String {
    text.chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_uppercase().next().unwrap() } else { c })
        .collect()
}

let cap_tool = quick_tool! {
    name: capitalize,
    description: "Capitalize the first letter of text",
    params: (text: String),
    execute: |text| capitalize(text)
};
```

### Example 3: Boolean Logic

```rust
fn greet(name: String, formal: bool) -> String {
    if formal {
        format!("Good day, {}", name)
    } else {
        format!("Hey {}!", name)
    }
}

let greet_tool = quick_tool! {
    name: greet,
    description: "Greet someone",
    params: (name: String, formal: bool),
    execute: |name, formal| greet(name, formal)
};
```

### Example 4: Complex Calculations

```rust
fn calculate_mortgage(principal: f64, rate: f64, years: i32) -> f64 {
    let monthly_rate = rate / 12.0 / 100.0;
    let num_payments = years * 12;
    let monthly_payment = principal * 
        (monthly_rate * (1.0 + monthly_rate).powi(num_payments)) /
        ((1.0 + monthly_rate).powi(num_payments) - 1.0);
    monthly_payment
}

let mortgage_tool = quick_tool! {
    name: calculate_mortgage,
    description: "Calculate monthly mortgage payment",
    params: (principal: f64, rate: f64, years: i32),
    execute: |principal, rate, years| {
        let payment = calculate_mortgage(principal, rate, years);
        format!(
            "Loan: ${:.2}, Rate: {:.2}%, Term: {} years\nMonthly payment: ${:.2}",
            principal, rate, years, payment
        )
    }
};
```

## Complete Working Example

```rust
use helios_engine::{quick_tool, Agent, Config};

// 1. Define your functions
fn area_rectangle(length: f64, width: f64) -> f64 {
    length * width
}

fn area_circle(radius: f64) -> f64 {
    std::f64::consts::PI * radius * radius
}

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    // 2. Wrap them with quick_tool!
    let rect_tool = quick_tool! {
        name: area_rectangle,
        description: "Calculate area of a rectangle",
        params: (length: f64, width: f64),
        execute: |length, width| {
            format!("Area: {:.2} square units", area_rectangle(length, width))
        }
    };

    let circle_tool = quick_tool! {
        name: area_circle,
        description: "Calculate area of a circle",
        params: (radius: f64),
        execute: |radius| {
            format!("Area: {:.2} square units", area_circle(radius))
        }
    };

    // 3. Add to agent
    let mut agent = Agent::builder("GeometryHelper")
        .config(config)
        .tool(rect_tool)
        .tool(circle_tool)
        .build()
        .await?;

    // 4. Use it!
    let response = agent.chat("What's the area of a rectangle 5x3?").await?;
    println!("{}", response);

    Ok(())
}
```

## Alternative Methods

If you prefer more control or need async functions, you can use the `ToolBuilder` methods:

### 1. `parameters()` Method

Define all parameters at once using a compact format:

```rust
ToolBuilder::new("calculate_volume")
    .description("Calculate the volume of a box")
    .parameters("width:f64:The width, height:f64:The height, depth:f64:The depth")
    .sync_function(|args| { /* ... */ })
    .build()
```

**Format**: `"name:type:description, name2:type2:description2, ..."`

**Supported Types**:
- Integer types: `i32`, `i64`, `u32`, `u64`, `isize`, `usize`, `integer` → maps to `"integer"`
- Float types: `f32`, `f64`, `number` → maps to `"number"`
- String types: `str`, `String`, `string` → maps to `"string"`
- Boolean types: `bool`, `boolean` → maps to `"boolean"`
- Complex types: `object`, `array` → maps to `"object"` or `"array"`

### 2. `from_fn()` Method

Create a tool directly from a function with all metadata in one place:

```rust
fn calculate_area(length: f64, width: f64) -> f64 {
    length * width
}

let tool = ToolBuilder::from_fn(
    "calculate_area",
    "Calculate the area of a rectangle",
    "length:f64:The length, width:f64:The width",
    |args| {
        let length = args.get("length").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let width = args.get("width").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let area = calculate_area(length, width);
        Ok(ToolResult::success(format!("Area: {}", area)))
    }
).build();
```

### 3. `from_async_fn()` Method

Same as `from_fn()` but for async functions:

```rust
async fn fetch_temperature(city: &str) -> Result<f64, String> {
    // API call here
    Ok(20.5)
}

let tool = ToolBuilder::from_async_fn(
    "fetch_temperature",
    "Get the current temperature for a city",
    "city:string:The name of the city",
    |args| async move {
        let city = args.get("city").and_then(|v| v.as_str()).unwrap_or("");
        let temp = fetch_temperature(city).await.unwrap_or(0.0);
        Ok(ToolResult::success(format!("Temperature: {}°C", temp)))
    }
).build();
```

## Advanced Usage

### Type Mapping Details

The parser automatically maps Rust types to JSON schema types:

| Input Types | Maps To |
|-------------|---------|
| `i32`, `i64`, `u32`, `u64`, `isize`, `usize`, `integer` | `"integer"` |
| `f32`, `f64`, `number` | `"number"` |
| `str`, `String`, `string` | `"string"` |
| `bool`, `boolean` | `"boolean"` |
| `object` | `"object"` |
| `array` | `"array"` |

### Whitespace Handling

The parameter parser is forgiving with whitespace:

```rust
// All of these work:
"a:i32:First,b:i32:Second"
"a:i32:First, b:i32:Second"
"a:i32:First  ,  b:i32:Second"
"  a  :  i32  :  First  ,  b  :  i32  :  Second  "
```

### Mixing Builder Methods

You can mix the old and new APIs if needed:

```rust
let tool = ToolBuilder::new("process_data")
    .description("Process data")
    .parameters("name:string:Name, age:i32:Age")  // Bulk parameters
    .optional_parameter("email", "string", "Email address")  // Individual optional parameter
    .sync_function(|args| {
        // implementation
    })
    .build();
```

## Tips and Best Practices

### 1. Keep Functions Pure

Your functions should be simple and focused:

```rust
// Good - pure calculation
fn calculate_tax(amount: f64, rate: f64) -> f64 {
    amount * rate
}

// Better to handle formatting in execute block
let tool = quick_tool! {
    name: calculate_tax,
    description: "Calculate tax amount",
    params: (amount: f64, rate: f64),
    execute: |amount, rate| {
        let tax = calculate_tax(amount, rate);
        format!("Tax: ${:.2}", tax)
    }
};
```

### 2. Use the Execute Block for Formatting

The execute block is where you format the output:

```rust
let tool = quick_tool! {
    name: divide,
    description: "Divide two numbers",
    params: (a: f64, b: f64),
    execute: |a, b| {
        if b == 0.0 {
            "Error: Cannot divide by zero".to_string()
        } else {
            format!("Result: {:.2}", divide(a, b))
        }
    }
};
```

### 3. Reuse Functions Across Tools

One function can be used in multiple tools:

```rust
fn convert_temp(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}

let simple_convert = quick_tool! {
    name: celsius_to_fahrenheit,
    description: "Convert Celsius to Fahrenheit",
    params: (celsius: f64),
    execute: |c| format!("{:.1}°F", convert_temp(c))
};

let detailed_convert = quick_tool! {
    name: detailed_temp_convert,
    description: "Detailed temperature conversion",
    params: (celsius: f64),
    execute: |c| {
        let f = convert_temp(c);
        format!("{:.1}°C = {:.1}°F", c, f)
    }
};
```

## Migration Guide

### Converting Existing Tools

**Before (Manual Implementation)**:
```rust
struct AddTool;

#[async_trait]
impl Tool for AddTool {
    fn name(&self) -> &str { "add" }
    fn description(&self) -> &str { "Add two numbers" }
    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert("a".to_string(), ToolParameter {
            param_type: "integer".to_string(),
            description: "First number".to_string(),
            required: Some(true),
        });
        params.insert("b".to_string(), ToolParameter {
            param_type: "integer".to_string(),
            description: "Second number".to_string(),
            required: Some(true),
        });
        params
    }
    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let a = args.get("a").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let b = args.get("b").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        Ok(ToolResult::success(format!("{}", a + b)))
    }
}
```

**Lines of code: ~25**

**After (quick_tool! macro)**:
```rust
fn add(a: i32, b: i32) -> i32 { a + b }

let add_tool = quick_tool! {
    name: add,
    description: "Add two numbers",
    params: (a: i32, b: i32),
    execute: |a, b| format!("{}", add(a, b))
};
```

**Lines of code: ~7**

### Converting ToolBuilder Code

**Before**:
```rust
let tool = ToolBuilder::new("calculate_volume")
    .description("Calculate the volume of a box")
    .required_parameter("width", "number", "The width")
    .required_parameter("height", "number", "The height")
    .required_parameter("depth", "number", "The depth")
    .sync_function(|args| {
        // implementation
    })
    .build();
```

**After (Option 1 - using `parameters()`)**:
```rust
let tool = ToolBuilder::new("calculate_volume")
    .description("Calculate the volume of a box")
    .parameters("width:f64:The width, height:f64:The height, depth:f64:The depth")
    .sync_function(|args| {
        // implementation
    })
    .build();
```

**After (Option 2 - using `from_fn()`)**:
```rust
let tool = ToolBuilder::from_fn(
    "calculate_volume",
    "Calculate the volume of a box",
    "width:f64:The width, height:f64:The height, depth:f64:The depth",
    |args| {
        // implementation
    }
).build();
```

## Run the Examples

Try the complete example to see tools in action:

```bash
cargo run --example tool_builder_demo
```

This will show you working examples of the tool creation API.

## Comparison: Why Use quick_tool!?

### Readability

**Old Way**: 5 lines just for parameters
```rust
.required_parameter("width", "number", "The width")
.required_parameter("height", "number", "The height")
.required_parameter("depth", "number", "The depth")
```

**New Way**: 1 line for all parameters
```rust
params: (width: f64, height: f64, depth: f64)
```

### Less Boilerplate

**Old Way**: ~25 lines for a simple tool
```rust
struct MyTool;
#[async_trait]
impl Tool for MyTool {
    // ... lots of boilerplate
}
```

**New Way**: ~7 lines for the same tool
```rust
let tool = quick_tool! {
    name: my_function,
    description: "What it does",
    params: (x: i32, y: i32),
    execute: |x, y| format!("Result: {}", x + y)
};
```

### Better Alignment with Function Signatures

When you have an existing function:

```rust
fn calculate_volume(width: f64, height: f64, depth: f64) -> f64 {
    width * height * depth
}
```

The macro mirrors the function signature naturally:

```rust
quick_tool! {
    name: calculate_volume,
    description: "Calculate volume",
    params: (width: f64, height: f64, depth: f64),  // Matches function params
    execute: |width, height, depth| {
        format!("Volume: {:.2}", calculate_volume(width, height, depth))
    }
}
```

## Summary

Creating tools in Helios Engine is now as simple as:

1. **Write your function** - Normal Rust function, nothing special
2. **Wrap it with `quick_tool!`** - One macro call with name, description, params, and logic
3. **Add to agent** - Use `.tool(your_tool)` when building your agent

No boilerplate. No complexity. Just your logic wrapped and ready to use!

## See Also

- [TOOLS.md](TOOLS.md) - Complete tools documentation
- [Examples](../examples/) - Working code examples
- [API Documentation](https://docs.rs/helios-engine) - Full API reference