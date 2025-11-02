# Helios Engine Examples

This directory contains comprehensive examples demonstrating various features of the Helios Engine framework.

## Table of Contents

- [Running Examples](#running-examples)
- [Basic Examples](#basic-examples)
- [Agent Examples](#agent-examples)
- [Advanced Examples](#advanced-examples)
- [API Examples](#api-examples)

## Running Examples

All examples can be run using Cargo:

```bash
# Run a specific example
cargo run --example basic_chat

# List all available examples
cargo run --example --list
```

### Individual Example Commands

```bash
# Basic chat example
cargo run --example basic_chat

# Agent with built-in tools (Calculator, Echo)
cargo run --example agent_with_tools

# Agent with file management tools
cargo run --example agent_with_file_tools

# Agent with in-memory database tool
cargo run --example agent_with_memory_db

# Custom tool implementation
cargo run --example custom_tool

# Multiple agents with different personalities
cargo run --example multiple_agents

# Forest of Agents - collaborative multi-agent system
cargo run --example forest_of_agents

# Direct LLM usage without agents
cargo run --example direct_llm_usage

# Streaming chat with remote models
cargo run --example streaming_chat

# Local model streaming example
cargo run --example local_streaming

# Serve an agent via HTTP API
cargo run --example serve_agent

# Serve with custom endpoints
cargo run --example serve_with_custom_endpoints

# Complete demo with all features
cargo run --example complete_demo
```

## Basic Examples

### Basic Chat (`basic_chat.rs`)

The simplest way to use Helios Engine - direct chat with an LLM:

```rust
use helios_engine::{Agent, Config};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    let mut agent = Agent::builder("Assistant")
        .config(config)
        .system_prompt("You are a helpful assistant.")
        .build()
        .await?;

    let response = agent.chat("Hello!").await?;
    println!("{}", response);

    Ok(())
}
```

### Direct LLM Usage (`direct_llm_usage.rs`)

Use the LLM client directly without agents:

```rust
use helios_engine::{LLMClient, ChatMessage, llm::LLMProviderType};
use helios_engine::config::LLMConfig;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let llm_config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY").unwrap(),
        temperature: 0.7,
        max_tokens: 2048,
    };

    let client = LLMClient::new(LLMProviderType::Remote(llm_config)).await?;

    let messages = vec![
        ChatMessage::system("You are a helpful assistant."),
        ChatMessage::user("What is Rust?"),
    ];

    let response = client.chat(messages, None).await?;
    println!("Response: {}", response.content);

    Ok(())
}
```

## Agent Examples

### Agent with Tools (`agent_with_tools.rs`)

Create an agent with built-in calculator and echo tools:

```rust
use helios_engine::{Agent, Config, CalculatorTool, EchoTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    let mut agent = Agent::builder("ToolAgent")
        .config(config)
        .system_prompt("You have access to tools. Use them wisely.")
        .tool(Box::new(CalculatorTool))
        .tool(Box::new(EchoTool))
        .max_iterations(5)
        .build()
        .await?;

    // The agent will automatically use the calculator
    let response = agent.chat("What is 123 * 456?").await?;
    println!("{}", response);

    Ok(())
}
```

### Multiple Agents (`multiple_agents.rs`)

Run multiple agents with different personalities:

```rust
use helios_engine::{Agent, Config};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    let mut poet = Agent::builder("Poet")
        .config(config.clone())
        .system_prompt("You are a creative poet.")
        .build()
        .await?;

    let mut scientist = Agent::builder("Scientist")
        .config(config)
        .system_prompt("You are a knowledgeable scientist.")
        .build()
        .await?;

    let poem = poet.chat("Write a haiku about code").await?;
    let fact = scientist.chat("Explain quantum physics").await?;

    println!("Poet: {}\n", poem);
    println!("Scientist: {}", fact);

    Ok(())
}
```

### Forest of Agents (`forest_of_agents.rs`)

Create a collaborative multi-agent system where agents can communicate, delegate tasks, and share context:

```rust
use helios_engine::{Agent, Config, ForestBuilder};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    // Create a forest with specialized agents
    let forest = ForestBuilder::new()
        .config(config)
        .agent(
            "coordinator".to_string(),
            Agent::builder("coordinator")
                .system_prompt("You coordinate team projects and delegate tasks.")
        )
        .agent(
            "researcher".to_string(),
            Agent::builder("researcher")
                .system_prompt("You research and analyze information.")
        )
        .build()
        .await?;

    // Execute collaborative tasks
    let result = forest
        .execute_collaborative_task(
            &"coordinator".to_string(),
            "Create a guide on sustainable practices".to_string(),
            vec!["researcher".to_string()],
        )
        .await?;

    println!("Collaborative result: {}", result);

    // Direct inter-agent communication
    forest
        .send_message(
            &"coordinator".to_string(),
            Some(&"researcher".to_string()),
            "Please research the latest findings.".to_string(),
        )
        .await?;

    Ok(())
}
```

**Features:**
- **Multi-agent collaboration** on complex tasks
- **Inter-agent communication** (direct messages and broadcasts)
- **Task delegation** between agents
- **Shared context** and memory
- **Specialized agent roles** working together

### File Management Agent (`agent_with_file_tools.rs`)

Agent with comprehensive file management capabilities:

```rust
use helios_engine::{Agent, Config, FileSearchTool, FileReadTool, FileWriteTool, FileEditTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    let mut agent = Agent::builder("FileAssistant")
        .config(config)
        .system_prompt("You are a helpful file management assistant.")
        .tool(Box::new(FileSearchTool))
        .tool(Box::new(FileReadTool))
        .tool(Box::new(FileWriteTool))
        .tool(Box::new(FileEditTool))
        .build()
        .await?;

    // Set session memory
    agent.set_memory("session_start", chrono::Utc::now().to_rfc3339());
    agent.set_memory("working_directory", std::env::current_dir()?.display().to_string());

    // Use file tools
    let response = agent.chat("Find all Rust files in the src directory").await?;
    println!("Agent: {}\n", response);

    // Track tasks
    agent.increment_tasks_completed();

    // Get session summary
    println!("{}", agent.get_session_summary());

    Ok(())
}
```

### Memory Database Agent (`agent_with_memory_db.rs`)

Agent with in-memory database for data persistence:

```rust
use helios_engine::{Agent, Config, MemoryDBTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    let mut agent = Agent::builder("DataAgent")
        .config(config)
        .system_prompt("You can store and retrieve data using the memory_db tool.")
        .tool(Box::new(MemoryDBTool::new()))
        .build()
        .await?;

    // Store data
    agent.chat("Remember that my favorite color is blue").await?;

    // Agent automatically uses the database to remember
    agent.chat("What's my favorite color?").await?;
    // Response: "Your favorite color is blue"

    // Cache expensive computations
    agent.chat("Calculate 12345 * 67890 and save it as 'result'").await?;
    agent.chat("What was the result I asked you to calculate?").await?;

    // List all cached data
    let response = agent.chat("Show me everything you've stored").await?;
    println!("{response}");

    Ok(())
}
```

### Custom Tool (`custom_tool.rs`)

Create and use a custom tool:

```rust
use async_trait::async_trait;
use helios_engine::{Tool, ToolParameter, ToolResult};
use serde_json::Value;
use std::collections::HashMap;

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
                description: "City name".to_string(),
                required: Some(true),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> helios_engine::Result<ToolResult> {
        let location = args["location"].as_str().unwrap_or("Unknown");

        // Your weather API logic here
        let weather = format!("Weather in {}: Sunny, 72°F", location);

        Ok(ToolResult::success(weather))
    }
}

// Use your custom tool
#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    let mut agent = Agent::builder("WeatherAgent")
        .config(config)
        .tool(Box::new(WeatherTool))
        .build()
        .await?;

    let response = agent.chat("What's the weather in Tokyo?").await?;
    println!("{}", response);

    Ok(())
}
```

## Advanced Examples

### RAG with Qdrant (`rag_advanced.rs`)

Advanced RAG implementation with vector search:

```rust
// This example demonstrates using QdrantRAGTool for document retrieval
// Requires Qdrant running locally
use helios_engine::{Agent, Config, QdrantRAGTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let rag_tool = QdrantRAGTool::new(
        "http://localhost:6333",                    // Qdrant URL
        "my_collection",                             // Collection name
        "https://api.openai.com/v1/embeddings",     // Embedding API
        std::env::var("OPENAI_API_KEY").unwrap(),   // API key
    );

    let config = Config::from_file("config.toml")?;
    let mut agent = Agent::builder("RAGAgent")
        .config(config)
        .tool(Box::new(rag_tool))
        .build()
        .await?;

    // Add documents to the knowledge base
    agent.chat("Add this document about Rust: 'Rust is a systems programming language...'")
        .await?;

    // Query with semantic search
    let response = agent.chat("What is Rust programming?").await?;
    println!("{}", response);

    Ok(())
}
```

### Complete Demo (`complete_demo.rs`)

Showcase of all major features in one example.

## API Examples

### Serve Agent (`serve_agent.rs`)

Expose an agent via OpenAI-compatible HTTP API:

```rust
use helios_engine::{Agent, Config, CalculatorTool, serve};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    let agent = Agent::builder("API Agent")
        .config(config)
        .system_prompt("You are a helpful AI assistant with access to a calculator tool.")
        .tool(Box::new(CalculatorTool))
        .max_iterations(5)
        .build()
        .await?;

    println!("Starting server on http://127.0.0.1:8000");
    println!("Try: curl http://127.0.0.1:8000/v1/chat/completions \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -d '{{\"model\": \"local-model\", \"messages\": [{{\"role\": \"user\", \"content\": \"What is 15 * 7?\"}}]}}'");

    serve::start_server_with_agent(agent, "local-model".to_string(), "127.0.0.1:8000").await?;

    Ok(())
}
```

### Serve with Custom Endpoints (`serve_with_custom_endpoints.rs`)

Serve agents with additional custom API endpoints.

### Streaming Chat (`streaming_chat.rs`)

Real-time streaming responses:

```rust
use helios_engine::{LLMClient, ChatMessage, llm::LLMProviderType};
use helios_engine::config::LLMConfig;
use std::io::Write;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let llm_config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY").unwrap(),
        temperature: 0.7,
        max_tokens: 2048,
    };

    let client = LLMClient::new(LLMProviderType::Remote(llm_config)).await?;

    let messages = vec![
        ChatMessage::system("You are a helpful assistant that responds concisely."),
        ChatMessage::user("Write a short poem about programming."),
    ];

    println!("🤖: ");
    let response = client
        .chat_stream(messages, None, |chunk| {
            print!("{}", chunk);
            std::io::stdout().flush().unwrap(); // For immediate output
        })
        .await?;
    println!(); // New line after streaming completes

    Ok(())
}
```

### Local Streaming (`local_streaming.rs`)

Streaming with local models.

## Configuration

Most examples require a `config.toml` file. Create one based on your needs:

```toml
[llm]
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "your-api-key-here"
temperature = 0.7
max_tokens = 2048

# Optional: Add local configuration for offline mode
[local]
huggingface_repo = "unsloth/Qwen3-0.6B-GGUF"
model_file = "Qwen3-0.6B-Q4_K_M.gguf"
temperature = 0.7
max_tokens = 2048
```

## Prerequisites

- Rust 1.70+
- API keys for remote models (OpenAI, etc.)
- For local models: HuggingFace account and models
- For RAG examples: Qdrant vector database running locally

## Running All Examples

```bash
# Run all examples (one by one)
for example in $(cargo run --example --list | grep -E '^ ' | tr -d ' '); do
    echo "Running $example..."
    cargo run --example $example
done
```

Each example is self-contained and demonstrates specific functionality. Check the source code for detailed implementation and configuration options.
