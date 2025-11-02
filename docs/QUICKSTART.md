# Quick Start Guide

Get started with Helios Engine in 5 minutes! This guide assumes you have already installed Helios Engine. If not, see the **[Installation Guide](INSTALLATION.md)** first.

## Step 1: Configuration

### CLI Tool Quick Start

If you installed the CLI tool, initialize configuration:

```bash
# Create default config file
helios-engine init

# Start chatting immediately
helios-engine chat
```

### Library Crate Quick Start

Create a `config.toml` file:

```toml
[llm]
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-api-key-here"
temperature = 0.7
max_tokens = 2048
```

For other providers, see **[Configuration Guide](USAGE.md#configuration)**.

## Step 2: Your First Agent (Library Usage)

Create `main.rs`:

```rust
use helios_engine::{Agent, Config};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Load configuration
    let config = Config::from_file("config.toml")?;

    // Create your first agent
    let mut agent = Agent::builder("MyAgent")
        .config(config)
        .system_prompt("You are a helpful assistant.")
        .build()
        .await?;

    // Start chatting!
    let response = agent.chat("Hello! What can you do?").await?;
    println!("Agent: {}", response);

    Ok(())
}
```

Run it:

```bash
cargo run
```

## Step 3: Add Tools to Your Agent

```rust
use helios_engine::{Agent, Config, CalculatorTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    // Create agent with built-in calculator tool
    let mut agent = Agent::builder("ToolAgent")
        .config(config)
        .system_prompt("You have access to tools. Use them to help users!")
        .tool(Box::new(CalculatorTool))
        .build()
        .await?;

    // Agent can now perform calculations
    let response = agent.chat("What is 123 * 456?").await?;
    println!("Agent: {}", response);

    Ok(())
}
```

## Step 4: Try Advanced Features

### Forest of Agents (Multi-agent Collaboration)

```rust
use helios_engine::{Agent, Config, ForestBuilder};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    // Create a forest with specialized agents
    let mut forest = ForestBuilder::new()
        .config(config)
        .agent(
            "researcher".to_string(),
            Agent::builder("researcher")
                .system_prompt("You research and analyze information.")
        )
        .agent(
            "writer".to_string(),
            Agent::builder("writer")
                .system_prompt("You create content and documentation.")
        )
        .build()
        .await?;

    // Execute collaborative tasks
    let result = forest
        .execute_collaborative_task(
            &"researcher".to_string(),
            "Create a guide on sustainable practices".to_string(),
            vec!["writer".to_string()],
        )
        .await?;

    println!("Collaborative result: {}", result);
    Ok(())
}
```

### RAG System (Retrieval-Augmented Generation)

```rust
use helios_engine::{Agent, Config, RAGTool, InMemoryVectorStore, OpenAIEmbeddings};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    // Create RAG system
    let embeddings = OpenAIEmbeddings::new(
        "https://api.openai.com/v1/embeddings".to_string(),
        std::env::var("OPENAI_API_KEY").unwrap(),
    );

    let vector_store = InMemoryVectorStore::new(embeddings);
    let rag_tool = RAGTool::new(vector_store);

    // Create agent with RAG capabilities
    let mut agent = Agent::builder("RAGAgent")
        .config(config)
        .system_prompt("You have access to a knowledge base.")
        .tool(Box::new(rag_tool))
        .build()
        .await?;

    // Add documents and query
    agent.chat("Add this document about Rust: 'Rust is a systems programming language...'").await?;
    let response = agent.chat("What is Rust programming?").await?;
    println!("Response: {}", response);

    Ok(())
}
```

## Next Steps

🎯 **Ready to dive deeper?**

1. **[Examples](../examples/)** - See Helios in action with complete working examples
2. **[Tools Guide](USAGE.md#tools)** - Learn about all built-in tools and how to create custom ones
3. **[Configuration Guide](USAGE.md#configuration)** - Set up local models and advanced configurations
4. **[API Reference](API.md)** - Complete technical documentation

## Troubleshooting

### Common Issues

**"Failed to read config file"**
- Ensure `config.toml` exists in your project root
- Check file permissions
- Verify TOML syntax is valid

**"LLM API request failed"**
- Verify your API key is correct and has proper permissions
- Check that the base_url is accessible
- Ensure the model_name is valid for your provider

**"Tool execution failed"**
- Check tool parameter format in your request
- Verify all required parameters are provided
- Review the tool's documentation for correct usage

**Build errors with `local` feature**
- Ensure you have a C++ compiler installed
- Try building without the `local` feature first
- See **[Installation Guide](INSTALLATION.md)** for system requirements

## Need Help?

- 📖 **[Full Documentation](README.md)** - Main project documentation
- 💬 **Issues** - [GitHub Issues](https://github.com/Ammar-Alnagar/Helios-Engine/issues)
- 📚 **API Docs** - [docs.rs/helios-engine](https://docs.rs/helios-engine)

Happy coding with Helios Engine! 🚀
