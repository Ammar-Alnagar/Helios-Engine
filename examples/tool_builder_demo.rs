//! # Example: Tool Builder Demo
//!
//! This example demonstrates the SIMPLEST way to create custom tools.
//! Just define your function and pass it directly with ftool!
//! The system automatically infers types from your function signature!

use helios_engine::{Agent, Config, ToolBuilder};

// Your regular Rust functions - nothing special needed!
// The system automatically infers the types!

fn adder(x: i32, y: i32) -> i32 {
    x + y
}

fn multiplier(a: i32, b: i32) -> i32 {
    a * b
}

fn calculate_area(length: f64, width: f64) -> f64 {
    length * width
}

fn calculate_volume(width: f64, height: f64, depth: f64) -> f64 {
    width * height * depth
}

fn calculate_bmi(weight_kg: f64, height_m: f64) -> f64 {
    weight_kg / (height_m * height_m)
}

// Mixed types - String and bool!
fn greet(name: String, formal: bool) -> String {
    if formal {
        format!("Good day, {}. It's a pleasure to meet you.", name)
    } else {
        format!("Hey {}! What's up?", name)
    }
}

// Different numeric types
fn calculate_discount(price: f64, percent: i32) -> String {
    let discount = price * (percent as f64 / 100.0);
    format!(
        "Discount: ${:.2}, Final price: ${:.2}",
        discount,
        price - discount
    )
}

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    println!("=== Tool Builder Demo ===\n");
    println!("Creating tools with the ultra-simple ftool API!");
    println!("The system automatically infers types from your function signature!\n");

    // Example 1: Integer parameters (i32, i32) - types inferred automatically!
    let add_tool = ToolBuilder::new("add")
        .description("Add two integers")
        .parameters("x:i32:First number, y:i32:Second number")
        .ftool(adder) // Automatically knows these are i32!
        .build();

    // Example 2: Integer multiplication (i32, i32)
    let multiply_tool = ToolBuilder::new("multiply")
        .description("Multiply two integers")
        .parameters("a:i32:First number, b:i32:Second number")
        .ftool(multiplier)
        .build();

    // Example 3: Float parameters (f64, f64) - automatically inferred!
    let area_tool = ToolBuilder::new("calculate_area")
        .description("Calculate the area of a rectangle")
        .parameters("length:f64:Length in meters, width:f64:Width in meters")
        .ftool(calculate_area) // Same ftool method, but knows these are f64!
        .build();

    // Example 4: Three float parameters (f64, f64, f64)
    let volume_tool = ToolBuilder::new("calculate_volume")
        .description("Calculate the volume of a box")
        .parameters(
            "width:f64:Width in meters, height:f64:Height in meters, depth:f64:Depth in meters",
        )
        .ftool3(calculate_volume)
        .build();

    // Example 5: BMI calculator with floats
    let bmi_tool = ToolBuilder::new("calculate_bmi")
        .description("Calculate Body Mass Index")
        .parameters("weight_kg:f64:Weight in kilograms, height_m:f64:Height in meters")
        .ftool(calculate_bmi)
        .build();

    // Example 6: Mixed types - String and bool!
    let greet_tool = ToolBuilder::new("greet")
        .description("Greet someone by name")
        .parameters("name:string:Person's name, formal:bool:Use formal greeting")
        .ftool(greet) // Automatically knows: String, bool!
        .build();

    // Example 7: Mixed numeric types - f64 and i32
    let discount_tool = ToolBuilder::new("calculate_discount")
        .description("Calculate discount on a price")
        .parameters("price:f64:Original price, percent:i32:Discount percentage")
        .ftool(calculate_discount) // Automatically knows: f64, i32!
        .build();

    // Create an agent with all the tools
    let mut agent = Agent::builder("ToolDemo")
        .config(config)
        .system_prompt(
            "You are a helpful assistant with access to various calculation and utility tools. \
             Use them to help answer questions accurately.",
        )
        .tool(add_tool)
        .tool(multiply_tool)
        .tool(area_tool)
        .tool(volume_tool)
        .tool(bmi_tool)
        .tool(greet_tool)
        .tool(discount_tool)
        .build()
        .await?;

    println!("Created 7 tools with minimal code!\n");
    println!("All types automatically inferred from function signatures!\n");
    println!("===========================================\n");

    // Test the tools
    println!("Test 1: Integer addition");
    let response = agent.chat("What is 42 plus 17?").await?;
    println!("Agent: {}\n", response);

    println!("Test 2: Integer multiplication");
    let response = agent.chat("What is 12 times 8?").await?;
    println!("Agent: {}\n", response);

    println!("Test 3: Calculate area");
    let response = agent
        .chat("What is the area of a rectangle that is 5 meters long and 3 meters wide?")
        .await?;
    println!("Agent: {}\n", response);

    println!("Test 4: Calculate volume");
    let response = agent
        .chat("What's the volume of a box that is 2m wide, 3m high, and 1.5m deep?")
        .await?;
    println!("Agent: {}\n", response);

    println!("Test 5: Calculate BMI");
    let response = agent
        .chat("Calculate BMI for someone weighing 70 kg and 1.75 meters tall")
        .await?;
    println!("Agent: {}\n", response);

    println!("Test 6: Greeting with String and bool");
    let response = agent.chat("Greet Bob in a casual way").await?;
    println!("Agent: {}\n", response);

    println!("Test 7: Discount calculation with mixed types");
    let response = agent
        .chat("What's the discount on a $100 item with 20% off?")
        .await?;
    println!("Agent: {}\n", response);

    println!("\n===========================================");
    println!("✨ That's how easy it is to create tools!");
    println!("   Just define your function and the system automatically:");
    println!("   • Infers all parameter types from your function signature");
    println!("   • Extracts values from JSON in the right order");
    println!("   • Handles i32, i64, u32, u64, f32, f64, bool, String");
    println!(
        "   • Works with .ftool() for 2 params, .ftool3() for 3 params, .ftool4() for 4 params"
    );

    Ok(())
}
