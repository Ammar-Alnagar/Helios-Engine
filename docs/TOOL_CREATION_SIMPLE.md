# Simple Tool Creation Guide

This guide shows you the **simplest possible way** to create tools in Helios Engine.

## The Philosophy

You already have functions. We just need to wrap them. That's it!

## Quick Start

### Step 1: Write Your Function

```rust
fn adder(x: i32, y: i32) -> i32 {
    x + y
}
```

### Step 2: Wrap It

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

### Step 3: Use It

```rust
let mut agent = Agent::builder("MyAgent")
    .config(config)
    .tool(tool)
    .build()
    .await?;
```

That's it! No boilerplate, no complex trait implementations, no manual JSON parsing.

## The `quick_tool!` Macro

The `quick_tool!` macro does all the heavy lifting:

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

## Examples

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

## Supported Types

The macro supports these types out of the box:

| Rust Type | JSON Type | Default Value |
|-----------|-----------|---------------|
| `i32`, `i64`, `u32`, `u64` | integer | `0` |
| `f32`, `f64` | number | `0.0` |
| `bool` | boolean | `false` |
| `String` | string | `""` |

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

## Comparison: Old vs New

### Old Way (Manual Implementation)

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

### New Way (quick_tool! macro)

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

## Run the Examples

Try the complete example:

```bash
cargo run --example tool_ultimate_simple
```

This will show you working examples of the simple tool creation API.

## Summary

Creating tools in Helios Engine is now as simple as:

1. **Write your function** - Normal Rust function, nothing special
2. **Wrap it with `quick_tool!`** - One macro call with name, description, params, and logic
3. **Add to agent** - Use `.tool(your_tool)` when building your agent

No boilerplate. No complexity. Just your logic wrapped and ready to use!
