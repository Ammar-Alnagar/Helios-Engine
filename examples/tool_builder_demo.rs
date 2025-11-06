//! # Example: Tool Builder Demo
//!
//! This example demonstrates the SIMPLEST way to create custom tools.
//! Just define your function and pass it directly with ftool!

use helios_engine::{Agent, Config, ToolBuilder};

// Your regular Rust functions - nothing special needed!
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

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    println!("=== Tool Builder Demo ===\n");
    println!("Creating tools with the ultra-simple ftool API!\n");

    // Example 1: Simple integer addition - just pass the function!
    let add_tool = ToolBuilder::new("add")
        .description("Add two integers")
        .parameters("x:i32:First number, y:i32:Second number")
        .ftool(adder)
        .build();

    // Example 2: Integer multiplication
    let multiply_tool = ToolBuilder::new("multiply")
        .description("Multiply two integers")
        .parameters("a:i32:First number, b:i32:Second number")
        .ftool(multiplier)
        .build();

    // Example 3: Float calculation - area
    let area_tool = ToolBuilder::new("calculate_area")
        .description("Calculate the area of a rectangle")
        .parameters("length:f64:Length in meters, width:f64:Width in meters")
        .ftool_f64(calculate_area)
        .build();

    // Example 4: Three-parameter float calculation - volume
    let volume_tool = ToolBuilder::new("calculate_volume")
        .description("Calculate the volume of a box")
        .parameters(
            "width:f64:Width in meters, height:f64:Height in meters, depth:f64:Depth in meters",
        )
        .ftool3_f64(calculate_volume)
        .build();

    // Example 5: BMI calculator with float parameters
    let bmi_tool = ToolBuilder::new("calculate_bmi")
        .description("Calculate Body Mass Index")
        .parameters("weight_kg:f64:Weight in kilograms, height_m:f64:Height in meters")
        .ftool_f64(|weight, height| {
            let bmi = calculate_bmi(weight, height);
            let category = match bmi {
                b if b < 18.5 => format!("{:.1} (Underweight)", b),
                b if b < 25.0 => format!("{:.1} (Normal weight)", b),
                b if b < 30.0 => format!("{:.1} (Overweight)", b),
                b => format!("{:.1} (Obese)", b),
            };
            category
        })
        .build();

    // Create an agent with all the tools
    let mut agent = Agent::builder("ToolDemo")
        .config(config)
        .system_prompt(
            "You are a helpful assistant with access to various calculation tools. \
             Use them to help answer questions accurately.",
        )
        .tool(add_tool)
        .tool(multiply_tool)
        .tool(area_tool)
        .tool(volume_tool)
        .tool(bmi_tool)
        .build()
        .await?;

    println!("Created 5 tools with minimal code!\n");
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

    println!("\n===========================================");
    println!("✨ That's how easy it is to create tools!");
    println!("   Just define your function and use:");
    println!("   .ftool(your_function) for i32 parameters");
    println!("   .ftool_f64(your_function) for f64 parameters");
    println!("   .ftool3_f64(your_function) for 3 f64 parameters");

    Ok(())
}
