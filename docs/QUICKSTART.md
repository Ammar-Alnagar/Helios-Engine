# Quick Start Guide

Get started with Helios in 5 minutes!

## Step 1: Installation

Clone the repository:
```bash
git clone https://github.com/yourusername/helios.git
cd helios
```

Or add to your project:
```toml
[dependencies]
helios = "0.1.0"
tokio = { version = "1.35", features = ["full"] }
```

## Step 2: Configuration

Create `config.toml` in your project root:

```toml
[llm]
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-api-key-here"
temperature = 0.7
max_tokens = 2048
```

### For Different Providers

**OpenAI:**
```toml
[llm]
model_name = "gpt-4"
base_url = "https://api.openai.com/v1"
api_key = "sk-..."
```

**Local LM Studio:**
```toml
[llm]
model_name = "local-model"
base_url = "http://localhost:1234/v1"
api_key = "not-needed"
```

**Ollama:**
```toml
[llm]
model_name = "llama2"
base_url = "http://localhost:11434/v1"
api_key = "not-needed"
```

## Step 3: Your First Agent

Create `main.rs`:

```rust
use helios::{Agent, Config};

#[tokio::main]
async fn main() -> helios::Result<()> {
    // Load config
    let config = Config::from_file("config.toml")?;
    
    // Create agent
    let mut agent = Agent::builder("MyAgent")
        .config(config)
        .system_prompt("You are a helpful assistant.")
        .build()?;
    
    // Chat!
    let response = agent.chat("Hello! What can you do?").await?;
    println!("Agent: {}", response);
    
    Ok(())
}
```

## Step 4: Run It

```bash
cargo run
```

## Step 5: Add Tools

```rust
use helios::{Agent, Config, CalculatorTool};

#[tokio::main]
async fn main() -> helios::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let mut agent = Agent::builder("ToolAgent")
        .config(config)
        .system_prompt("You have access to tools. Use them!")
        .tool(Box::new(CalculatorTool))
        .build()?;
    
    let response = agent.chat("What is 123 * 456?").await?;
    println!("Agent: {}", response);
    
    Ok(())
}
```

## Common Patterns

### Interactive Chat Loop

```rust
use std::io::{self, Write};

loop {
    print!("You: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if input.trim() == "exit" {
        break;
    }
    
    let response = agent.chat(input.trim()).await?;
    println!("Agent: {}\n", response);
}
```

### Error Handling

```rust
match agent.chat("Hello").await {
    Ok(response) => println!("Success: {}", response),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Multiple Agents

```rust
let mut agent1 = Agent::builder("Agent1")
    .config(config.clone())
    .system_prompt("You are a math expert.")
    .build()?;

let mut agent2 = Agent::builder("Agent2")
    .config(config)
    .system_prompt("You are a creative writer.")
    .build()?;
```

## Next Steps

1. Check out the [examples](../examples/)
2. Read the [Architecture Guide](../ARCHITECTURE.md)
3. Create your own [custom tools](../README.md#creating-custom-tools)
4. Join the community and contribute!

## Troubleshooting

### "Failed to read config file"
- Ensure `config.toml` exists
- Check file permissions
- Verify TOML syntax

### "LLM API request failed"
- Verify API key is correct
- Check base_url is accessible
- Ensure model_name is valid

### "Tool execution failed"
- Check tool parameter format
- Verify required parameters provided
- Review tool implementation

## Resources

- [Full Documentation](../README.md)
- [API Reference](https://docs.rs/helios)
- [Examples](../examples/)
- [Contributing](../CONTRIBUTING.md)

Happy coding! 🚀
