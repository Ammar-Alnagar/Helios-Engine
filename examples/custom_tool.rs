use async_trait::async_trait;
use helios_engine::{Agent, Config, Tool, ToolParameter, ToolResult};
use serde_json::Value;
use std::collections::HashMap;

// Define a custom tool
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
                description: "The city and state, e.g. San Francisco, CA".to_string(),
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
        let location = args
            .get("location")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        let unit = args
            .get("unit")
            .and_then(|v| v.as_str())
            .unwrap_or("fahrenheit");

        // Simulate weather data (in a real implementation, call a weather API)
        let temp = if unit == "celsius" { "22" } else { "72" };
        let weather = format!(
            "The weather in {} is sunny with a temperature of {}°{}",
            location,
            temp,
            if unit == "celsius" { "C" } else { "F" }
        );

        Ok(ToolResult::success(weather))
    }
}

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Load configuration
    let config = Config::from_file("config.toml")?;

    // Create an agent with custom tool
    let mut agent = Agent::builder("WeatherAgent")
        .config(config)
        .system_prompt("You are a helpful weather assistant. Use the weather tool to answer questions about weather.")
        .tool(Box::new(WeatherTool))
        .build()
        .await?;

    // Ask about weather
    let response = agent.chat("What's the weather like in New York?").await?;
    println!("Agent: {}\n", response);

    let response = agent.chat("How about in London, but in celsius?").await?;
    println!("Agent: {}\n", response);

    Ok(())
}
