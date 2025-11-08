# 🔥 Helios Engine - LLM Agent Framework

<p align="center">
  <img src="src/Helios_Engine_Logo.png" alt="Helios Engine Logo" width="350"/>
</p>

<p align="center">
  <a href="https://crates.io/crates/helios-engine"><img src="https://img.shields.io/crates/v/helios-engine.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/helios-engine"><img src="https://img.shields.io/badge/docs-latest-blue.svg" alt="Documentation"></a>
  <a href="https://helios-engine.vercel.app/"><img src="https://img.shields.io/badge/book-online-brightgreen.svg" alt="Online Book"></a>
  <a href="https://github.com/Ammar-Alnagar/helios-engine/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
</p>

**Helios Engine** is a powerful and flexible Rust framework for building LLM-powered agents with tool support, streaming chat capabilities, and easy configuration management. Create intelligent agents that can interact with users, call tools, and maintain conversation context - with both online and offline local model support.

## 📚 Documentation

- **[Official Documentation & Book](https://helios-engine.vercel.app/)** - Complete guide with tutorials and examples
- **[API Reference](https://docs.rs/helios-engine)** - Detailed API documentation on docs.rs
- **[Examples](https://github.com/Ammar-Alnagar/helios-engine/tree/main/examples)** - Hands-on code examples

## 🚀 Key Features

### Core Capabilities
- **🤖 Agent System**: Create multiple agents with different personalities, system prompts, and specialized capabilities
- **🔧 Tool Registry**: Extensible tool system for adding custom functionality to your agents
- **💬 Streaming Support**: True real-time response streaming for both remote and local models with immediate token delivery
- **🌐 HTTP Server & API**: Expose fully OpenAI-compatible API endpoints with streaming, temperature control, and all standard parameters
- **🎯 Dual Mode Support**: Auto, online (remote API), and offline (local) modes for maximum flexibility

### Advanced Features
- **🌲 Forest of Agents**: Multi-agent collaboration system where agents can communicate, delegate tasks, and share context
  - Coordinator-based planning for complex task decomposition
  - Agent-to-agent communication
  - Shared context and memory between agents
  
- **🧰 Tool Builder**: Simplified tool creation with builder pattern - wrap any function as a tool without manual trait implementation

- **📚 RAG System (Retrieval-Augmented Generation)**: Built-in support for semantic search and document retrieval
  - Vector stores: InMemory and Qdrant
  - OpenAI embeddings integration
  - Easy document storage and retrieval

- **🛠️ Extensive Tool Suite**: 16+ built-in tools including:
  - **File Management**: Read, write, edit, and search files
  - **Web Tools**: Web scraping, HTTP requests
  - **System Tools**: Shell commands, system information
  - **Data Processing**: JSON parsing, text manipulation, timestamps
  - **Utilities**: Calculator, echo, in-memory database

- **🧩 Feature Flags**: Optional `local` feature for offline model support - build only what you need!
  - Standard installation for lightweight deployments
  - Local model support with llama.cpp integration

- **📦 CLI & Library**: Use as both a command-line tool and a Rust library crate

## 📦 Installation

### Version 0.4.4

#### As a CLI Tool

**Standard Installation** (recommended for most users):
```bash
cargo install helios-engine
```

**With Local Model Support** (includes llama.cpp for offline models):
```bash
cargo install helios-engine --features local
```

#### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
helios-engine = "0.4.4"
tokio = { version = "1.35", features = ["full"] }
```

For local model support:
```toml
[dependencies]
helios-engine = { version = "0.4.4", features = ["local"] }
tokio = { version = "1.35", features = ["full"] }
```

## 🚦 Quick Start

### Basic Agent Example

```rust
use helios_engine::{Agent, Config};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let mut agent = Agent::builder("MyAssistant")
        .config(config)
        .system_prompt("You are a helpful AI assistant.")
        .build()
        .await?;
    
    let response = agent.chat("Hello! How are you?").await?;
    println!("{}", response);
    
    Ok(())
}
```

### Agent with Tools

```rust
use helios_engine::{Agent, Config, CalculatorTool, FileReadTool, WebScraperTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let mut agent = Agent::builder("ToolAgent")
        .config(config)
        .system_prompt("You are a helpful assistant with access to various tools.")
        .tools(vec![
            Box::new(CalculatorTool),
            Box::new(FileReadTool),
            Box::new(WebScraperTool),
        ])
        .max_iterations(5)
        .build()
        .await?;
    
    let response = agent.chat("What is 15 * 7 + 23?").await?;
    println!("{}", response);
    
    Ok(())
}
```

### Forest of Agents (Multi-Agent System)

```rust
use helios_engine::{Agent, Config, ForestBuilder};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let mut forest = ForestBuilder::new()
        .config(config)
        .agents(vec![
            ("researcher".to_string(), Agent::builder("researcher")
                .system_prompt("You research and gather information.")),
            ("analyst".to_string(), Agent::builder("analyst")
                .system_prompt("You analyze data and identify patterns.")),
            ("writer".to_string(), Agent::builder("writer")
                .system_prompt("You write clear, concise reports.")),
        ])
        .max_iterations(15)
        .build()
        .await?;
    
    let result = forest.execute("Research AI trends and write a summary").await?;
    println!("{}", result);
    
    Ok(())
}
```

### RAG (Retrieval-Augmented Generation)

```rust
use helios_engine::{Agent, Config, RAGTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let rag_tool = RAGTool::new_in_memory(
        "https://api.openai.com/v1/embeddings",
        std::env::var("OPENAI_API_KEY").unwrap()
    );
    
    let mut agent = Agent::builder("KnowledgeAgent")
        .config(config)
        .tool(Box::new(rag_tool))
        .build()
        .await?;
    
    // Store documents
    agent.chat("Store this: Rust is a systems programming language focused on safety and performance.").await?;
    
    // Query knowledge base
    let response = agent.chat("What do you know about Rust?").await?;
    println!("{}", response);
    
    Ok(())
}
```

### HTTP API Server

```rust
use helios_engine::{Agent, Config, CalculatorTool, serve};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let agent = Agent::builder("API Agent")
        .config(config)
        .system_prompt("You are a helpful AI assistant with calculator capabilities.")
        .tool(Box::new(CalculatorTool))
        .max_iterations(5)
        .build()
        .await?;
    
    println!("Starting server on http://127.0.0.1:8000");
    serve::start_server_with_agent(agent, "gpt-4".to_string(), "127.0.0.1:8000").await?;
    
    Ok(())
}
```

Test with curl:
```bash
curl http://127.0.0.1:8000/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "What is 15 * 7?"}],
    "stream": true
  }'
```

## 🔧 Configuration

Create a `config.toml` file:

```toml
# Online mode (using remote API)
[llm]
mode = "online"
provider = "openai"
model = "gpt-4"
api_url = "https://api.openai.com/v1/chat/completions"
api_key = "your-api-key-here"

# Optional parameters
temperature = 0.7
max_tokens = 2000
streaming = true
```

For local models:
```toml
[llm]
mode = "offline"
model_path = "/path/to/model.gguf"
```

Or use auto mode to let Helios decide:
```toml
[llm]
mode = "auto"
provider = "openai"
model = "gpt-4"
```

## 📖 Built-in Tools

Helios Engine includes 16+ production-ready tools:

### File Management
- **FileReadTool** - Read file contents with line range support
- **FileWriteTool** - Write content to files
- **FileEditTool** - Find and replace in files
- **FileSearchTool** - Search files by name or content

### Web & API
- **WebScraperTool** - Fetch and extract web content
- **HttpRequestTool** - Make HTTP requests (GET, POST, PUT, DELETE, etc.)

### System & Utilities
- **ShellCommandTool** - Execute shell commands with safety restrictions
- **SystemInfoTool** - Get system information (OS, CPU, memory, disk, network)
- **CalculatorTool** - Perform mathematical calculations
- **TimestampTool** - Work with timestamps and dates

### Data Processing
- **JsonParseTool** - Parse and manipulate JSON data
- **TextProcessingTool** - Text manipulation and analysis
- **InMemoryDbTool** - Simple key-value database in memory

### Agent Communication
- **SendMessageTool** - Enable agent-to-agent communication in Forest mode

### RAG & Knowledge
- **RAGTool** - Retrieval-augmented generation with vector stores

## 🎯 Use Cases

- **Chatbots & Virtual Assistants**: Build conversational AI with tool access
- **Multi-Agent Systems**: Coordinate multiple specialized agents for complex workflows
- **Data Analysis**: Agents that can read files, process data, and generate reports
- **Web Automation**: Scrape websites, make API calls, and process responses
- **Knowledge Management**: Build RAG systems for semantic search and Q&A
- **API Services**: Expose your agents via OpenAI-compatible HTTP endpoints
- **Local AI**: Run models completely offline for privacy and security

## 🌟 Advanced Features

### Streaming Responses
```rust
use futures::StreamExt;

let mut stream = agent.chat_stream("Tell me a story").await?;
while let Some(chunk) = stream.next().await {
    print!("{}", chunk?);
}
```

### Custom Tools
```rust
use helios_engine::{Tool, ToolBuilder};
use async_trait::async_trait;
use serde_json::{json, Value};

// Using Tool Builder (recommended)
let custom_tool = ToolBuilder::new("my_tool")
    .description("Does something custom")
    .parameter("input", "string", "Input parameter", true)
    .build(|params| async move {
        let input = params.get("input").and_then(|v| v.as_str()).unwrap_or("");
        Ok(json!({"result": format!("Processed: {}", input)}))
    });

// Or implement the Tool trait manually
struct MyCustomTool;

#[async_trait]
impl Tool for MyCustomTool {
    fn name(&self) -> String {
        "my_custom_tool".to_string()
    }
    
    fn description(&self) -> String {
        "My custom tool description".to_string()
    }
    
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "input": {"type": "string", "description": "Input parameter"}
            },
            "required": ["input"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        // Your custom logic here
        Ok(json!({"result": "success"}))
    }
}
```

### Coordinator-Based Planning
```rust
let mut forest = ForestBuilder::new()
    .config(config)
    .coordinator("coordinator".to_string(), 
        Agent::builder("coordinator")
            .system_prompt("You plan and coordinate tasks."))
    .agents(vec![
        ("worker1".to_string(), Agent::builder("worker1")),
        ("worker2".to_string(), Agent::builder("worker2")),
    ])
    .max_iterations(20)
    .build()
    .await?;
```

## 📊 Examples

The repository includes extensive examples demonstrating all features:

```bash
# List all examples
cargo run --example --list

# Run specific examples
cargo run --example basic_chat
cargo run --example agent_with_tools
cargo run --example forest_of_agents
cargo run --example agent_with_rag
cargo run --example streaming_chat
cargo run --example serve_agent
```

Check out the [examples directory](https://github.com/Ammar-Alnagar/helios-engine/tree/main/examples) and the [online documentation](https://helios-engine.vercel.app/examples/overview.html) for more.

## 🤝 Contributing

Contributions are welcome! Please see our [Contributing Guide](https://helios-engine.vercel.app/contributing/how_to_contribute.html) for details.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🔗 Links

- **[Official Website & Documentation](https://helios-engine.vercel.app/)**
- **[Crates.io](https://crates.io/crates/helios-engine)**
- **[API Documentation](https://docs.rs/helios-engine)**
- **[GitHub Repository](https://github.com/Ammar-Alnagar/helios-engine)**

## 🎉 Getting Help

- Check out the [online book](https://helios-engine.vercel.app/) for comprehensive guides
- Browse the [examples](https://github.com/Ammar-Alnagar/helios-engine/tree/main/examples) for code samples
- Open an issue on [GitHub](https://github.com/Ammar-Alnagar/helios-engine/issues) for bugs or feature requests

---

<p align="center">
  Made with ❤️ by <a href="https://github.com/Ammar-Alnagar">Ammar Alnagar</a>
</p>
