//! # Example: Tool Builder Demo
//!
//! This example demonstrates the SIMPLE way to create custom tools.
//! Just write your function and wrap it with quick_tool! - that's it!

use helios_engine::{quick_tool, Agent, Config};

// Your regular Rust functions - nothing special needed!
fn calculate_area(length: f64, width: f64) -> f64 {
    length * width
}

fn calculate_volume(width: f64, height: f64, depth: f64) -> f64 {
    width * height * depth
}

fn calculate_bmi(weight_kg: f64, height_m: f64) -> f64 {
    weight_kg / (height_m * height_m)
}

fn greet(name: String, formal: bool) -> String {
    if formal {
        format!("Good day, {}. It's a pleasure to meet you.", name)
    } else {
        format!("Hey {}! What's up?", name)
    }
}

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    println!("=== Tool Builder Demo ===\n");
    println!("Creating tools is now as simple as wrapping your functions!\n");

    // Just wrap your functions with quick_tool! - that's it!
    let area_tool = quick_tool! {
        name: calculate_area,
        description: "Calculate the area of a rectangle",
        params: (length: f64, width: f64),
        execute: |length, width| {
            let area = calculate_area(length, width);
            format!("The area is {:.2} square units", area)
        }
    };

    let volume_tool = quick_tool! {
        name: calculate_volume,
        description: "Calculate the volume of a box",
        params: (width: f64, height: f64, depth: f64),
        execute: |width, height, depth| {
            let volume = calculate_volume(width, height, depth);
            format!("The volume is {:.2} cubic units", volume)
        }
    };

    let bmi_tool = quick_tool! {
        name: calculate_bmi,
        description: "Calculate Body Mass Index",
        params: (weight_kg: f64, height_m: f64),
        execute: |weight, height| {
            let bmi = calculate_bmi(weight, height);
            let category = match bmi {
                b if b < 18.5 => "Underweight",
                b if b < 25.0 => "Normal weight",
                b if b < 30.0 => "Overweight",
                _ => "Obese",
            };
            format!("BMI: {:.1} ({})", bmi, category)
        }
    };

    let greet_tool = quick_tool! {
        name: greet,
        description: "Greet someone by name",
        params: (name: String, formal: bool),
        execute: |name, formal| {
            greet(name, formal)
        }
    };

    // Create an agent with all the tools
    let mut agent = Agent::builder("ToolBuilderDemo")
        .config(config)
        .system_prompt(
            "You are a helpful assistant with access to various tools. \
             Use them to help answer questions accurately.",
        )
        .tool(area_tool)
        .tool(volume_tool)
        .tool(bmi_tool)
        .tool(greet_tool)
        .build()
        .await?;

    println!("Created 4 tools with minimal code!\n");
    println!("===========================================\n\n");

    // Test the tools
    println!("Test 1: Calculate area");
    let response = agent
        .chat("What is the area of a rectangle that is 5 meters long and 3 meters wide?")
        .await?;
    println!("Agent: {}\n", response);

    println!("Test 2: Calculate volume");
    let response = agent
        .chat(
            "What's the volume of a box that is 2 meters wide, 3 meters high, and 1.5 meters deep?",
        )
        .await?;
    println!("Agent: {}\n", response);

    println!("Test 3: Calculate BMI");
    let response = agent
        .chat("Calculate BMI for someone weighing 70 kg and 1.75 meters tall")
        .await?;
    println!("Agent: {}\n", response);

    println!("Test 4: Greeting");
    let response = agent.chat("Greet Alice in a casual way").await?;
    println!("Agent: {}\n", response);

    println!("\n===========================================");
    println!("That's how easy it is to create tools!");
    println!("Just wrap your functions with quick_tool! and you're done!");

    Ok(())
}
