# Building Your First Agent

Agents are the core of the Helios Engine. They are autonomous entities that can use tools to accomplish tasks. Here's how to build your first agent:

```rust
use helios_engine::{Agent, Config, CalculatorTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Load the configuration from a config.toml file
    let config = Config::from_file("config.toml")?;

    // Create a new agent using the AgentBuilder
    let mut agent = Agent::builder("MathAgent")
        .config(config)
        .system_prompt("You are a helpful math assistant.")
        .tool(Box::new(CalculatorTool))
        .max_iterations(5)
        .build()
        .await?;

    // Chat with the agent
    let response = agent.chat("What is 15 * 8 + 42?").await?;
    println!("Agent: {}", response);

    Ok(())
}
```

In this example, we create an agent named "MathAgent" with a system prompt that tells it how to behave. We also give it a `CalculatorTool`, which allows it to perform mathematical calculations. When we ask the agent to solve a math problem, it will automatically use the calculator to find the answer.
