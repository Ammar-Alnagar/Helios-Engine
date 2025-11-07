# Getting Started with Helios Engine

A comprehensive guide to get you up and running with Helios Engine quickly.

## Table of Contents
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Basic Usage](#basic-usage)
- [Building Your First Agent](#building-your-first-agent)
- [Working with Tools](#working-with-tools)
- [Forest of Agents](#forest-of-agents)
- [Next Steps](#next-steps)

## Installation

### As a CLI Tool
```bash
cargo install helios-engine
```

### As a Library
Add to your `Cargo.toml`:
```toml
[dependencies]
helios-engine = "0.4"
```

## Quick Start

### 1. Initialize Configuration
```bash
helios-engine init
```

This creates a `config.toml` file. Edit it with your API key:

```toml
[llm]
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-api-key-here"
temperature = 0.7
max_tokens = 2048
```

### 2. Start Chatting
```bash
helios-engine chat
```

Or ask a one-off question:
```bash
helios-engine ask "What is Rust?"
```

## Basic Usage

### As a Library

#### Simple LLM Call
```rust
use helios_engine::{LLMClient, ChatMessage, Config};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    let client = LLMClient::new(config.llm);
    
    let messages = vec![ChatMessage::user("Hello, world!")];
    let response = client.chat(messages, None).await?;
    
    println!("Assistant: {}", response.content);
    Ok(())
}
```

#### Streaming Responses
```rust
use tokio_stream::StreamExt;

let mut stream = client.chat_stream(messages, None).await?;
while let Some(chunk) = stream.next().await {
    print!("{}", chunk?);
}
```

#### Conversation Management
```rust
use helios_engine::ChatSession;

let mut session = ChatSession::new()
    .with_system_prompt("You are a helpful coding assistant.");

session.add_user_message("Explain async/await in Rust");
let response = client.chat(session.get_messages(), None).await?;
session.add_assistant_message(&response.content);

// Continue the conversation
session.add_user_message("Can you give an example?");
let response2 = client.chat(session.get_messages(), None).await?;
```

## Building Your First Agent

Agents are autonomous entities that can use tools to accomplish tasks:

```rust
use helios_engine::{Agent, Config, CalculatorTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let mut agent = Agent::builder("MathAgent")
        .config(config)
        .system_prompt("You are a helpful math assistant.")
        .tool(Box::new(CalculatorTool))
        .max_iterations(5)
        .build()
        .await?;
    
    let response = agent.chat("What is 15 * 8 + 42?").await?;
    println!("Agent: {}", response);
    
    Ok(())
}
```

## Working with Tools

### Adding Multiple Tools (New Improved Syntax!)

**Old way** (still supported):
```rust
let agent = Agent::builder("MyAgent")
    .tool(Box::new(CalculatorTool))
    .tool(Box::new(EchoTool))
    .tool(Box::new(FileSearchTool))
    .build()
    .await?;
```

**New improved way**:
```rust
let agent = Agent::builder("MyAgent")
    .tools(vec![
        Box::new(CalculatorTool),
        Box::new(EchoTool),
        Box::new(FileSearchTool),
    ])
    .build()
    .await?;
```

### Built-in Tools

Helios Engine includes several built-in tools:

- **CalculatorTool** - Perform mathematical calculations
- **EchoTool** - Echo messages back
- **FileSearchTool** - Search for files by pattern or content
- **FileReadTool** - Read file contents
- **FileWriteTool** - Write to files
- **FileEditTool** - Find and replace in files

### Creating Custom Tools

```rust
use helios_engine::{Tool, tool_builder};
use async_trait::async_trait;
use serde_json::Value;

struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> String {
        "get_weather".to_string()
    }
    
    fn description(&self) -> String {
        "Get the current weather for a city".to_string()
    }
    
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "The city name"
                }
            },
            "required": ["city"]
        })
    }
    
    async fn execute(&self, args: Value) -> helios_engine::Result<String> {
        let city = args["city"].as_str().unwrap_or("Unknown");
        Ok(format!("The weather in {} is sunny, 72°F", city))
    }
}
```

## Forest of Agents

Forest of Agents allows multiple agents to collaborate on complex tasks:

### Basic Forest

```rust
use helios_engine::{Agent, Config, ForestBuilder};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let forest = ForestBuilder::new()
        .config(config)
        .agent("coordinator".to_string(), 
            Agent::builder("coordinator")
                .system_prompt("You coordinate tasks between agents."))
        .agent("worker1".to_string(),
            Agent::builder("worker1")
                .system_prompt("You are a helpful worker."))
        .build()
        .await?;
    
    // Use the forest...
    Ok(())
}
```

### Multiple Agents at Once (New Improved Syntax!)

```rust
let forest = ForestBuilder::new()
    .config(config)
    .agents(vec![
        ("coordinator".to_string(), Agent::builder("coordinator")
            .system_prompt("You coordinate tasks.")),
        ("worker1".to_string(), Agent::builder("worker1")
            .system_prompt("You handle data processing.")),
        ("worker2".to_string(), Agent::builder("worker2")
            .system_prompt("You handle analysis.")),
    ])
    .max_iterations(20)
    .build()
    .await?;
```

### Forest with Coordinator-Based Planning

Enable automatic task planning and delegation:

```rust
let forest = ForestBuilder::new()
    .config(config)
    .enable_coordinator_planning()
    .coordinator_agent("coordinator".to_string(),
        Agent::builder("coordinator")
            .system_prompt("You create and manage plans."))
    .agent("researcher".to_string(), 
        Agent::builder("researcher")
            .system_prompt("You research information."))
    .agent("writer".to_string(),
        Agent::builder("writer")
            .system_prompt("You write content."))
    .build()
    .await?;
```

## Next Steps

### Learn More
- [Complete API Documentation](API.md)
- [Tool Creation Guide](TOOLS.md)
- [Forest of Agents Guide](FOREST.md)
- [RAG (Retrieval Augmented Generation)](RAG.md)
- [Configuration Options](CONFIGURATION.md)

### Examples
Check out the `examples/` directory for more:
- `basic_chat.rs` - Simple chat example
- `agent_with_tools.rs` - Agent with tools
- `forest_simple_demo.rs` - Basic forest example
- `forest_with_coordinator.rs` - Advanced forest planning
- `streaming_chat.rs` - Streaming responses
- `rag_advanced.rs` - RAG implementation

### CLI Reference

#### Commands
```bash
helios-engine                    # Interactive chat (default)
helios-engine chat              # Interactive chat
helios-engine ask "question"    # One-off question
helios-engine init              # Create config file
helios-engine --help            # Show help
```

#### Options
```bash
-c, --config <FILE>      # Custom config file
-v, --verbose            # Verbose logging
-s, --system-prompt      # Custom system prompt
-m, --max-iterations     # Max tool iterations
```

#### Interactive Commands
- `exit`, `quit` - Exit chat
- `clear` - Clear conversation history
- `history` - Show conversation history
- `summary` - Show session summary
- `tools` - List available tools
- `help` - Show help

### Common Providers

**OpenAI:**
```toml
base_url = "https://api.openai.com/v1"
model_name = "gpt-4"
api_key = "sk-..."
```

**Local (LM Studio):**
```toml
base_url = "http://localhost:1234/v1"
model_name = "local-model"
api_key = "not-needed"
```

**Ollama:**
```toml
base_url = "http://localhost:11434/v1"
model_name = "llama2"
api_key = "not-needed"
```

**Anthropic:**
```toml
base_url = "https://api.anthropic.com/v1"
model_name = "claude-3-opus-20240229"
api_key = "sk-ant-..."
```

---

**Ready to build?** Start with the examples directory or dive into the [full documentation](README.md).
