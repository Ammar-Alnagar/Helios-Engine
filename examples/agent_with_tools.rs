use helios::{Agent, Config, CalculatorTool, EchoTool};

#[tokio::main]
async fn main() -> helios::Result<()> {
    // Load configuration
    let config = Config::from_file("config.toml")?;

    // Create an agent with tools
    let mut agent = Agent::builder("ToolAgent")
        .config(config)
        .system_prompt("You are a helpful assistant with access to tools. Use them when needed.")
        .tool(Box::new(CalculatorTool))
        .tool(Box::new(EchoTool))
        .max_iterations(5)
        .build()?;

    println!("Available tools: {:?}\n", agent.tool_registry().list_tools());

    // Test calculator tool
    let response = agent.chat("What is 25 * 4 + 10?").await?;
    println!("Agent: {}\n", response);

    // Test echo tool
    let response = agent.chat("Can you echo this message: 'Hello from Helios!'").await?;
    println!("Agent: {}\n", response);

    Ok(())
}
