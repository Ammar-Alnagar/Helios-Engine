use helios_engine::{Agent, Config, CalculatorTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Load configuration
    let config = Config::from_file("config.toml")?;

    // Create multiple agents with different personalities
    let mut math_agent = Agent::builder("MathAgent")
        .config(config.clone())
        .system_prompt("You are a math expert. You love numbers and equations.")
        .tool(Box::new(CalculatorTool))
        .build()?;

    let mut creative_agent = Agent::builder("CreativeAgent")
        .config(config)
        .system_prompt("You are a creative writer who loves storytelling and poetry.")
        .build()?;

    println!("=== Math Agent ===");
    let response = math_agent.chat("What is the square root of 144?").await?;
    println!("Math Agent: {}\n", response);

    println!("=== Creative Agent ===");
    let response = creative_agent.chat("Write a haiku about programming.").await?;
    println!("Creative Agent: {}\n", response);

    Ok(())
}
