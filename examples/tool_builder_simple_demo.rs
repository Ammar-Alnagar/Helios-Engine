//! # Example: Simplified Tool Builder Demo
//!
//! This example demonstrates the new simplified API for creating tools:
//! 1. Define all parameters at once using a compact format
//! 2. Use `from_fn` to automatically derive tool from function name
//!
//! This makes it much easier to create tools without repetitive code!

use helios_engine::{Agent, Config, ToolBuilder, ToolResult};

// Example 1: A simple function to calculate volume
fn calculate_volume(width: f64, height: f64, depth: f64) -> f64 {
    width * height * depth
}

// Example 2: Calculate BMI
fn calculate_bmi(weight_kg: f64, height_m: f64) -> f64 {
    weight_kg / (height_m * height_m)
}

// Example 3: Async function to simulate database lookup
async fn lookup_user(user_id: i32) -> Result<String, String> {
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    match user_id {
        1 => Ok("Alice Smith".to_string()),
        2 => Ok("Bob Johnson".to_string()),
        3 => Ok("Charlie Brown".to_string()),
        _ => Err(format!("User {} not found", user_id)),
    }
}

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Load configuration
    let config = Config::from_file("config.toml")?;

    println!("=== Simplified Tool Builder Demo ===\n");
    println!("This demo shows how easy it is to create tools with the new API:\n");

    // OLD WAY (still works, but verbose):
    println!("OLD WAY - Multiple method calls:");
    println!("  ToolBuilder::new(\"calculate_volume\")");
    println!("      .description(\"...\")");
    println!("      .required_parameter(\"width\", \"number\", \"...\")");
    println!("      .required_parameter(\"height\", \"number\", \"...\")");
    println!("      .required_parameter(\"depth\", \"number\", \"...\")");
    println!("      .sync_function(|args| {{ ... }})");
    println!("      .build()\n");

    // NEW WAY (much simpler!):
    println!("NEW WAY - All at once:");
    println!("  ToolBuilder::from_fn(");
    println!("      \"calculate_volume\",");
    println!("      \"Calculate volume of a box\",");
    println!("      \"width:f64:Width in meters, height:f64:Height in meters, depth:f64:Depth in meters\",");
    println!("      |args| {{ ... }}");
    println!("  ).build()\n");
    println!("Much cleaner and easier to read!\n");
    println!("===========================================\n\n");

    // Example 1: Using the new simplified API with from_fn
    let volume_tool = ToolBuilder::from_fn(
        "calculate_volume",
        "Calculate the volume of a box given width, height, and depth",
        "width:f64:The width of the box in meters, height:f64:The height of the box in meters, depth:f64:The depth of the box in meters",
        |args| {
            let width = args.get("width").and_then(|v| v.as_f64()).ok_or_else(|| {
                helios_engine::HeliosError::ToolError("Missing or invalid 'width'".to_string())
            })?;
            let height = args.get("height").and_then(|v| v.as_f64()).ok_or_else(|| {
                helios_engine::HeliosError::ToolError("Missing or invalid 'height'".to_string())
            })?;
            let depth = args.get("depth").and_then(|v| v.as_f64()).ok_or_else(|| {
                helios_engine::HeliosError::ToolError("Missing or invalid 'depth'".to_string())
            })?;
            let volume = calculate_volume(width, height, depth);
            Ok(ToolResult::success(format!(
                "The volume is {:.2} cubic meters",
                volume
            )))
        }
    ).build();

    // Example 2: Using parameters() method to define all params at once
    let bmi_tool = ToolBuilder::new("calculate_bmi")
        .description("Calculate Body Mass Index from weight and height")
        .parameters("weight_kg:f64:Weight in kilograms, height_m:f64:Height in meters")
        .sync_function(|args| {
            let weight = args
                .get("weight_kg")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| {
                    helios_engine::HeliosError::ToolError("Missing 'weight_kg'".to_string())
                })?;

            let height = args
                .get("height_m")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| {
                    helios_engine::HeliosError::ToolError("Missing 'height_m'".to_string())
                })?;

            let bmi = calculate_bmi(weight, height);
            let category = match bmi {
                b if b < 18.5 => "Underweight",
                b if b < 25.0 => "Normal weight",
                b if b < 30.0 => "Overweight",
                _ => "Obese",
            };

            Ok(ToolResult::success(format!(
                "BMI: {:.1} ({})",
                bmi, category
            )))
        })
        .build();

    // Example 3: Async function with from_async_fn
    let user_tool = ToolBuilder::from_async_fn(
        "lookup_user",
        "Look up a user by their ID",
        "user_id:i32:The ID of the user to look up",
        |args| async move {
            let user_id = args
                .get("user_id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| {
                    helios_engine::HeliosError::ToolError("Missing 'user_id'".to_string())
                })? as i32;

            match lookup_user(user_id).await {
                Ok(name) => Ok(ToolResult::success(format!("Found user: {}", name))),
                Err(e) => Ok(ToolResult::error(e)),
            }
        },
    )
    .build();

    // Example 4: Tool with mixed parameter types (i32, string, etc.)
    let order_tool = ToolBuilder::new("create_order")
        .description("Create a new order")
        .parameters("product:string:Product name, quantity:i32:Quantity to order, priority:boolean:Is this a priority order")
        .sync_function(|args| {
            let product = args.get("product").and_then(|v| v.as_str()).unwrap_or("Unknown");
            let quantity = args.get("quantity").and_then(|v| v.as_i64()).unwrap_or(1);
            let priority = args.get("priority").and_then(|v| v.as_bool()).unwrap_or(false);

            let priority_str = if priority { " [PRIORITY]" } else { "" };
            Ok(ToolResult::success(format!(
                "Order created: {} x {} {}",
                quantity, product, priority_str
            )))
        })
        .build();

    // Create an agent with all the tools
    let mut agent = Agent::builder("SimplifiedToolDemo")
        .config(config)
        .system_prompt(
            "You are a helpful assistant with access to various tools. \
             Use the tools to help answer questions accurately.",
        )
        .tool(volume_tool)
        .tool(bmi_tool)
        .tool(user_tool)
        .tool(order_tool)
        .build()
        .await?;

    // Test the tools
    println!("Test 1: Calculate volume");
    let response = agent
        .chat("What is the volume of a box that is 2.5 meters wide, 1.5 meters high, and 3 meters deep?")
        .await?;
    println!("Agent: {}\n", response);

    println!("Test 2: Calculate BMI");
    let response = agent
        .chat("Calculate the BMI for someone who weighs 70 kg and is 1.75 meters tall.")
        .await?;
    println!("Agent: {}\n", response);

    println!("Test 3: Look up user");
    let response = agent.chat("Look up user with ID 2").await?;
    println!("Agent: {}\n", response);

    println!("Test 4: Create order");
    let response = agent
        .chat("Create an order for 5 laptops, make it a priority order")
        .await?;
    println!("Agent: {}\n", response);

    Ok(())
}
