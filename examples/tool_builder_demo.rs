//! # Example: Tool Builder Demo
//!
//! This example demonstrates how to use the ToolBuilder to create custom tools
//! with a simplified API. Instead of implementing the Tool trait manually,
//! you can use the builder pattern to quickly create tools.

use helios_engine::{Agent, Config, ToolBuilder, ToolResult};
use serde_json::Value;

// Example 1: A simple synchronous function that we want to use as a tool
fn calculate_area(length: f64, width: f64) -> f64 {
    length * width
}

// Example 2: An async function that simulates an API call
async fn fetch_temperature(city: &str) -> Result<f64, String> {
    // Simulate API delay
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Mock temperature data
    match city.to_lowercase().as_str() {
        "london" => Ok(15.5),
        "paris" => Ok(18.2),
        "tokyo" => Ok(22.8),
        "new york" => Ok(20.1),
        _ => Err(format!("Unknown city: {}", city)),
    }
}

// Example 3: A more complex function with multiple parameters
fn format_currency(amount: f64, currency: &str, show_symbol: bool) -> String {
    let symbol = match currency.to_uppercase().as_str() {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" => "¥",
        _ => "",
    };

    if show_symbol && !symbol.is_empty() {
        format!("{}{:.2}", symbol, amount)
    } else {
        format!("{:.2} {}", amount, currency.to_uppercase())
    }
}

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Load configuration
    let config = Config::from_file("config.toml")?;

    // Example 1: Create a tool from a synchronous function
    let area_tool = ToolBuilder::new("calculate_area")
        .description("Calculate the area of a rectangle given length and width")
        .required_parameter("length", "number", "The length of the rectangle")
        .required_parameter("width", "number", "The width of the rectangle")
        .sync_function(|args: Value| {
            let length = args.get("length").and_then(|v| v.as_f64()).ok_or_else(|| {
                helios_engine::HeliosError::ToolError(
                    "Missing or invalid 'length' parameter".to_string(),
                )
            })?;

            let width = args.get("width").and_then(|v| v.as_f64()).ok_or_else(|| {
                helios_engine::HeliosError::ToolError(
                    "Missing or invalid 'width' parameter".to_string(),
                )
            })?;

            // Call our existing function
            let area = calculate_area(length, width);
            Ok(ToolResult::success(format!(
                "The area is {} square units",
                area
            )))
        })
        .build();

    // Example 2: Create a tool from an async function
    let weather_tool = ToolBuilder::new("get_temperature")
        .description("Get the current temperature for a city")
        .required_parameter("city", "string", "The name of the city")
        .function(|args: Value| async move {
            let city = args.get("city").and_then(|v| v.as_str()).ok_or_else(|| {
                helios_engine::HeliosError::ToolError("Missing 'city' parameter".to_string())
            })?;

            // Call our async function
            match fetch_temperature(city).await {
                Ok(temp) => Ok(ToolResult::success(format!(
                    "The temperature in {} is {:.1}°C",
                    city, temp
                ))),
                Err(e) => Ok(ToolResult::error(e)),
            }
        })
        .build();

    // Example 3: Create a tool with optional parameters
    let currency_tool = ToolBuilder::new("format_currency")
        .description("Format an amount as currency with optional symbol display")
        .required_parameter("amount", "number", "The amount to format")
        .optional_parameter("currency", "string", "Currency code (USD, EUR, GBP, JPY)")
        .optional_parameter("show_symbol", "boolean", "Whether to show currency symbol")
        .sync_function(|args: Value| {
            let amount = args.get("amount").and_then(|v| v.as_f64()).ok_or_else(|| {
                helios_engine::HeliosError::ToolError(
                    "Missing or invalid 'amount' parameter".to_string(),
                )
            })?;

            let currency = args
                .get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("USD");

            let show_symbol = args
                .get("show_symbol")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            let formatted = format_currency(amount, currency, show_symbol);
            Ok(ToolResult::success(formatted))
        })
        .build();

    // Example 4: Inline tool with closure capturing external state
    let discount_rate = 0.15; // 15% discount
    let discount_tool = ToolBuilder::new("apply_discount")
        .description("Apply a fixed discount to a price")
        .required_parameter("price", "number", "The original price")
        .sync_function(move |args: Value| {
            let price = args.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);

            let discounted = price * (1.0 - discount_rate);
            Ok(ToolResult::success(format!(
                "Original: ${:.2}, Discounted: ${:.2} ({}% off)",
                price,
                discounted,
                (discount_rate * 100.0) as i32
            )))
        })
        .build();

    // Create an agent with all the tools
    let mut agent = Agent::builder("ToolBuilderDemo")
        .config(config)
        .system_prompt(
            "You are a helpful assistant with access to various calculation tools. \
             Use the tools to help answer questions accurately.",
        )
        .tool(area_tool)
        .tool(weather_tool)
        .tool(currency_tool)
        .tool(discount_tool)
        .build()
        .await?;

    println!("=== Tool Builder Demo ===\n");

    // Test 1: Area calculation
    println!(
        "Question 1: What is the area of a rectangle that is 5 meters long and 3 meters wide?"
    );
    let response = agent
        .chat("What is the area of a rectangle that is 5 meters long and 3 meters wide?")
        .await?;
    println!("Agent: {}\n", response);

    // Test 2: Weather query
    println!("Question 2: What's the temperature in Tokyo?");
    let response = agent.chat("What's the temperature in Tokyo?").await?;
    println!("Agent: {}\n", response);

    // Test 3: Currency formatting
    println!("Question 3: Format 1234.56 as euros with the symbol.");
    let response = agent
        .chat("Format 1234.56 as euros with the symbol.")
        .await?;
    println!("Agent: {}\n", response);

    // Test 4: Discount calculation
    println!("Question 4: If a product costs $200, what would the discounted price be?");
    let response = agent
        .chat("If a product costs $200, what would the discounted price be?")
        .await?;
    println!("Agent: {}\n", response);

    Ok(())
}
