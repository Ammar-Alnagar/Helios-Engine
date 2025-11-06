# Simplified Tool Builder API

The Tool Builder has been enhanced with the `quick_tool!` macro and simplified methods that make creating tools incredibly easy. No more boilerplate, no more manual parameter extraction - just define what you want!

## Table of Contents

1. [Overview](#overview)
2. [The `quick_tool!` Macro](#the-quick_tool-macro) ⭐ **Recommended**
3. [Alternative Methods](#alternative-methods)
4. [Quick Examples](#quick-examples)
5. [Detailed Usage](#detailed-usage)
6. [Migration Guide](#migration-guide)

## Overview

The new simplified API provides three levels of simplicity:

1. **`quick_tool!` Macro** ⭐ **EASIEST** - Zero boilerplate, automatic parameter extraction
2. **`parameters()` Method** - Define all parameters in a single string
3. **`from_fn()` Method** - Derive tool from function name with parameters string

## The `quick_tool!` Macro

⭐ **This is the recommended and easiest way to create tools!**

The `quick_tool!` macro does everything for you:
- Automatically extracts parameters from JSON
- Maps Rust types to JSON schema types  
- Handles all boilerplate code
- One simple expression to create a complete tool

### Basic Usage

```rust
use helios_engine::quick_tool;

let tool = quick_tool! {
    name: calculate_area,
    description: "Calculate the area of a rectangle",
    params: (length: f64, width: f64),
    execute: |length, width| {
        format!("Area: {} square units", length * width)
    }
};
```

That's it! The tool is ready to use with your agent.

### Supported Types

- **Integers**: `i32`, `i64`, `u32`, `u64`
- **Floats**: `f32`, `f64`
- **Booleans**: `bool`
- **Strings**: `String`

The macro automatically handles:
- Type conversion from JSON
- Default values (0 for numbers, false for bools, empty string for strings)
- Error-free parameter extraction

### More Examples

**BMI Calculator:**
```rust
let bmi_tool = quick_tool! {
    name: calculate_bmi,
    description: "Calculate Body Mass Index",
    params: (weight_kg: f64, height_m: f64),
    execute: |weight, height| {
        let bmi = weight / (height * height);
        format!("BMI: {:.1}", bmi)
    }
};
```

**Greeting with Boolean:**
```rust
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

**Power Calculation:**
```rust
let power_tool = quick_tool! {
    name: calculate_power,
    description: "Calculate base raised to exponent",
    params: (base: f64, exponent: i32),
    execute: |base, exp| {
        format!("{} ^ {} = {:.2}", base, exp, base.powi(exp))
    }
};
```

## Alternative Methods

If you prefer more control or need async functions, you can use these methods:

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

## Quick Examples

### Example 1: Simple Calculator

**Old Way** (still works):
```rust
let tool = ToolBuilder::new("add")
    .description("Add two numbers")
    .required_parameter("a", "number", "First number")
    .required_parameter("b", "number", "Second number")
    .sync_function(|args| {
        let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
        Ok(ToolResult::success((a + b).to_string()))
    })
    .build();
```

**New Way** (much simpler):
```rust
let tool = ToolBuilder::from_fn(
    "add",
    "Add two numbers",
    "a:f64:First number, b:f64:Second number",
    |args| {
        let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
        Ok(ToolResult::success((a + b).to_string()))
    }
).build();
```

### Example 2: BMI Calculator

```rust
fn calculate_bmi(weight_kg: f64, height_m: f64) -> f64 {
    weight_kg / (height_m * height_m)
}

let bmi_tool = ToolBuilder::from_fn(
    "calculate_bmi",
    "Calculate Body Mass Index",
    "weight_kg:f64:Weight in kilograms, height_m:f64:Height in meters",
    |args| {
        let weight = args.get("weight_kg").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let height = args.get("height_m").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let bmi = calculate_bmi(weight, height);
        Ok(ToolResult::success(format!("BMI: {:.1}", bmi)))
    }
).build();
```

### Example 3: Order Processing

```rust
let order_tool = ToolBuilder::new("create_order")
    .description("Create a new order")
    .parameters("product:string:Product name, quantity:i32:Quantity, priority:bool:Priority order")
    .sync_function(|args| {
        let product = args.get("product").and_then(|v| v.as_str()).unwrap_or("Unknown");
        let quantity = args.get("quantity").and_then(|v| v.as_i64()).unwrap_or(1);
        let priority = args.get("priority").and_then(|v| v.as_bool()).unwrap_or(false);
        
        Ok(ToolResult::success(format!(
            "Order: {} x {} {}",
            quantity,
            product,
            if priority { "[PRIORITY]" } else { "" }
        )))
    })
    .build();
```

### Example 4: Async Database Lookup

```rust
async fn lookup_user(user_id: i32) -> Result<String, String> {
    // Database query here
    Ok("John Doe".to_string())
}

let user_tool = ToolBuilder::from_async_fn(
    "lookup_user",
    "Look up a user by their ID",
    "user_id:i32:The ID of the user",
    |args| async move {
        let id = args.get("user_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        match lookup_user(id).await {
            Ok(name) => Ok(ToolResult::success(format!("User: {}", name))),
            Err(e) => Ok(ToolResult::error(e)),
        }
    }
).build();
```

## Detailed Usage

### Parameter Format Details

The parameter string format is: `"name:type:description"`

- **Name**: The parameter name (required)
- **Type**: The data type (required)
- **Description**: Human-readable description (optional)

Multiple parameters are separated by commas.

**Examples**:

```rust
// With descriptions
"width:i32:The width in pixels, height:i32:The height in pixels"

// Without descriptions
"x:f64, y:f64"

// Mixed
"name:string:User name, age:i32, active:bool:Is user active"
```

### Type Mapping

The `parameters()` method automatically maps Rust types to JSON schema types:

| Input Types | Maps To |
|-------------|---------|
| `i32`, `i64`, `u32`, `u64`, `isize`, `usize`, `integer` | `"integer"` |
| `f32`, `f64`, `number` | `"number"` |
| `str`, `String`, `string` | `"string"` |
| `bool`, `boolean` | `"boolean"` |
| `object` | `"object"` |
| `array` | `"array"` |

### Whitespace Handling

The parser is forgiving with whitespace:

```rust
// All of these work:
"a:i32:First,b:i32:Second"
"a:i32:First, b:i32:Second"
"a:i32:First  ,  b:i32:Second"
"  a  :  i32  :  First  ,  b  :  i32  :  Second  "
```

### Empty Parameters

Empty parameter definitions are automatically skipped:

```rust
// Only 2 parameters will be created (a and b)
"a:i32:First, , b:i32:Second"
```

## Migration Guide

### Converting Existing Tools

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

### Mixing Old and New APIs

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

## Benefits

### Readability

**Before**: 5 lines just for parameters
```rust
.required_parameter("width", "number", "The width")
.required_parameter("height", "number", "The height")
.required_parameter("depth", "number", "The depth")
```

**After**: 1 line for all parameters
```rust
.parameters("width:f64:The width, height:f64:The height, depth:f64:The depth")
```

### Less Boilerplate

**Before**: Separate tool name definition
```rust
let tool = ToolBuilder::new("calculate_area")
    .description("Calculate area")
    .required_parameter("length", "number", "Length")
    .required_parameter("width", "number", "Width")
    .sync_function(|args| { /* ... */ })
    .build();
```

**After**: Everything in one place
```rust
let tool = ToolBuilder::from_fn(
    "calculate_area",
    "Calculate area",
    "length:f64:Length, width:f64:Width",
    |args| { /* ... */ }
).build();
```

### Better Alignment with Function Signatures

When you have an existing function:

```rust
fn calculate_volume(width: f64, height: f64, depth: f64) -> f64 {
    width * height * depth
}
```

The new API mirrors the function signature more closely:

```rust
ToolBuilder::from_fn(
    "calculate_volume",  // Function name
    "Calculate volume",
    "width:f64:Width, height:f64:Height, depth:f64:Depth",  // Matches function params
    |args| { /* wrapper code */ }
)
```

## Run the Example

Try the new simplified API with the provided example:

```bash
cargo run --example tool_builder_simple_demo
```

This example demonstrates all the new features and compares the old vs. new approaches.

## Backward Compatibility

All existing code continues to work. The new methods are additions, not replacements:

- `required_parameter()` - Still works
- `optional_parameter()` - Still works
- `parameter()` - Still works
- `new()` - Still works

You can adopt the new API gradually or stick with the old approach if you prefer.
