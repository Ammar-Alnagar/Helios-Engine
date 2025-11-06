//! # Example: Simple Tool Creation
//!
//! This example shows the EASIEST way to create tools with Helios Engine.
//! No boilerplate, no parameter extraction - just define what you want!

use helios_engine::{quick_tool, Agent, Config};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    println!("=== Simple Tool Creation Demo ===\n");
    println!("Creating tools is now as easy as writing a function!\n");

    // Example 1: Create a volume calculator tool - ONE EXPRESSION!
    let volume_tool = quick_tool! {
        name: calculate_volume,
        description: "Calculate the volume of a box",
        params: (width: f64, height: f64, depth: f64),
        execute: |width, height, depth| {
            format!("Volume: {:.2} cubic meters", width * height * depth)
        }
    };

    // Example 2: BMI calculator - SUPER SIMPLE!
    let bmi_tool = quick_tool! {
        name: calculate_bmi,
        description: "Calculate Body Mass Index",
        params: (weight_kg: f64, height_m: f64),
        execute: |weight, height| {
            let bmi = weight / (height * height);
            let category = match bmi {
                b if b < 18.5 => "Underweight",
                b if b < 25.0 => "Normal weight",
                b if b < 30.0 => "Overweight",
                _ => "Obese",
            };
            format!("BMI: {:.1} ({})", bmi, category)
        }
    };

    // Example 3: Temperature converter
    let temp_tool = quick_tool! {
        name: celsius_to_fahrenheit,
        description: "Convert Celsius to Fahrenheit",
        params: (celsius: f64),
        execute: |celsius| {
            let fahrenheit = (celsius * 9.0 / 5.0) + 32.0;
            format!("{:.1}°C = {:.1}°F", celsius, fahrenheit)
        }
    };

    // Example 4: String manipulation
    let greet_tool = quick_tool! {
        name: greet_user,
        description: "Greet a user with their name",
        params: (name: String, formal: bool),
        execute: |name, formal| {
            if formal {
                format!("Good day, {}. It's a pleasure to meet you.", name)
            } else {
                format!("Hey {}! What's up?", name)
            }
        }
    };

    // Example 5: Math operations
    let power_tool = quick_tool! {
        name: calculate_power,
        description: "Calculate base raised to exponent",
        params: (base: f64, exponent: i32),
        execute: |base, exp| {
            format!("{} ^ {} = {:.2}", base, exp, base.powi(exp))
        }
    };

    // Create an agent with all these tools
    let mut agent = Agent::builder("SimpleToolDemo")
        .config(config)
        .system_prompt(
            "You are a helpful assistant with access to various calculation tools. \
             Use them to help answer questions accurately.",
        )
        .tool(volume_tool)
        .tool(bmi_tool)
        .tool(temp_tool)
        .tool(greet_tool)
        .tool(power_tool)
        .build()
        .await?;

    println!("Created 5 tools with ZERO boilerplate!\n");
    println!("===========================================\n\n");

    // Test the tools
    println!("Test 1: Volume calculation");
    let response = agent
        .chat("What's the volume of a box that is 2.5 meters wide, 1.8 meters high, and 3.2 meters deep?")
        .await?;
    println!("Agent: {}\n", response);

    println!("Test 2: BMI calculation");
    let response = agent
        .chat("Calculate BMI for someone weighing 75 kg and 1.80 meters tall.")
        .await?;
    println!("Agent: {}\n", response);

    println!("Test 3: Temperature conversion");
    let response = agent
        .chat("Convert 25 degrees Celsius to Fahrenheit")
        .await?;
    println!("Agent: {}\n", response);

    println!("Test 4: Greeting");
    let response = agent.chat("Greet John in a formal way").await?;
    println!("Agent: {}\n", response);

    println!("Test 5: Power calculation");
    let response = agent.chat("What is 2 to the power of 10?").await?;
    println!("Agent: {}\n", response);

    println!("\n===========================================");
    println!("That's it! Creating tools is now SUPER EASY!");
    println!("Just use quick_tool! and you're done!");

    Ok(())
}
