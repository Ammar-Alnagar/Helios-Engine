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

Add to your `Cargo.toml`:
```toml
[dependencies]
helios-engine = "0.3.7"
tokio = { version = "1.35", features = ["full"] }
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
    println!("🚀 Helios Engine - Forest of Agents Demo");
    println!("=========================================\n");

    // Load configuration
    let config = Config::from_file("config.toml")?;

    // Create a Forest of Agents with specialized agents
    let mut forest = ForestBuilder::new()
        .config(config)
        // Coordinator agent - manages the team and delegates tasks
        .agent(
            "coordinator".to_string(),
            Agent::builder("coordinator")
                .system_prompt(
                    "You are a project coordinator responsible for breaking down complex tasks \
                    and delegating them to specialized team members. You communicate with other \
                    agents to ensure the project is completed successfully."
                )
        )
        // Research agent - gathers and analyzes information
        .agent(
            "researcher".to_string(),
            Agent::builder("researcher")
                .system_prompt(
                    "You are a research specialist who excels at gathering information, \
                    analyzing data, and providing insights."
                )
        )
        // Writer agent - creates content and documentation
        .agent(
            "writer".to_string(),
            Agent::builder("writer")
                .system_prompt(
                    "You are a skilled writer who creates clear, well-structured content and \
                    documentation. You work with the coordinator and researcher to produce \
                    high-quality written materials."
                )
        )
        .max_iterations(5)
        .build()
        .await?;

    println!("✓ Created Forest of Agents with 3 specialized agents\n");

    // Demonstrate collaborative task execution
    println!("🎯 Executing collaborative task:");
    println!("\"Create a comprehensive guide on sustainable gardening practices\"\n");

    let result = forest
        .execute_collaborative_task(
            &"coordinator".to_string(),
            "Create a comprehensive guide on sustainable gardening practices. This should include \
            environmental benefits, practical techniques, and tips for beginners.",
            vec![
                "researcher".to_string(),
                "writer".to_string(),
            ],
        )
        .await?;

    println!("📄 Final Result:");
    println!("{}", "=".repeat(60));
    println!("{}", result);
    println!("{}", "=".repeat(60));

    Ok(())
}
```

### RAG System (Retrieval-Augmented Generation)

```rust
use helios_engine::{Agent, Config, RAGTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Check for the required OpenAI API key
    let embedding_api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable must be set");

    // Load configuration
    let config = Config::from_file("config.toml")?;

    // Create a new RAG tool with in-memory vector store
    let rag_tool = RAGTool::new_in_memory(
        "https://api.openai.com/v1/embeddings",
        embedding_api_key
    );

    // Create agent with RAG capabilities
    let mut agent = Agent::builder("RAGAgent")
        .config(config)
        .system_prompt(
            "You are a helpful assistant with access to an in-memory RAG system. \
             You can store documents and retrieve relevant information to answer questions."
        )
        .tool(Box::new(rag_tool))
        .max_iterations(10)
        .build()
        .await?;

    // Add documents to knowledge base
    agent.chat("Store this information: Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.").await?;
    agent.chat("Store this information: Python is a high-level, interpreted programming language known for its clear syntax and readability.").await?;

    // Query the knowledge base
    let response = agent.chat("What programming language prevents segfaults?").await?;
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
