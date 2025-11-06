# Helios Engine Examples

This directory contains comprehensive examples demonstrating various features of the Helios Engine framework.

## Table of Contents

- [Running Examples](#running-examples)
- [Basic Examples](#basic-examples)
- [Agent Examples](#agent-examples)
- [Advanced Examples](#advanced-examples)
- [RAG Examples](#rag-examples)
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

# Forest with Coordinator - enhanced planning system
cargo run --example forest_with_coordinator

# Forest Simple Demo - simple reliable demo of planning system
cargo run --example forest_simple_demo

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

# SendMessageTool demo - test messaging functionality
cargo run --example send_message_tool_demo

# Agent with RAG capabilities
cargo run --example agent_with_rag

# RAG with in-memory vector store
cargo run --example rag_in_memory

# Compare RAG implementations (Qdrant vs InMemory)
cargo run --example rag_qdrant_comparison

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
    let mut forest = ForestBuilder::new()
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

### SendMessageTool Demo (`send_message_tool_demo.rs`)

Test the SendMessageTool functionality for agent communication:

```rust
use helios_engine::{Agent, Config, ForestBuilder, SendMessageTool, Tool};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    // Create a forest with two agents
    let forest = ForestBuilder::new()
        .config(config)
        .agent("alice".to_string(), Agent::builder("alice").build().await?)
        .agent("bob".to_string(), Agent::builder("bob").build().await?)
        .build()
        .await?;

    // Create SendMessageTool for direct testing
    let message_queue = Arc::new(RwLock::new(Vec::new()));
    let shared_context = Arc::new(RwLock::new(helios_engine::SharedContext::new()));

    let send_tool = SendMessageTool::new(
        "alice".to_string(),
        Arc::clone(&message_queue),
        Arc::clone(&shared_context),
    );

    // Test direct message
    let args = serde_json::json!({
        "to": "bob",
        "message": "Hello Bob!"
    });

    let result = send_tool.execute(args).await?;
    println!("Direct message result: {}", result.output);

    // Test broadcast message
    let args = serde_json::json!({
        "message": "Hello everyone!"
    });

    let result = send_tool.execute(args).await?;
    println!("Broadcast message result: {}", result.output);

    Ok(())
}
```

**Features:**
- **Direct messaging** between specific agents
- **Broadcast messaging** to all agents in the forest
- **Message queue management** and shared context integration
- **Forest messaging system integration**

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

### Tool Builder Demo (`tool_builder_demo.rs`)

**NEW!** Create custom tools easily with the ToolBuilder - no need to implement the Tool trait manually:

```rust
use helios_engine::{ToolBuilder, ToolResult};
use serde_json::Value;

// Wrap your existing function as a tool
let calculator = ToolBuilder::new("multiply")
    .description("Multiply two numbers")
    .required_parameter("x", "number", "First number")
    .required_parameter("y", "number", "Second number")
    .sync_function(|args: Value| {
        let x = args.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let y = args.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
        Ok(ToolResult::success((x * y).to_string()))
    })
    .build();
```

**✨ THE EASIEST WAY TO CREATE TOOLS!** Use the `ftool` API - just pass your function directly:

```rust
use helios_engine::ToolBuilder;

// Your normal function
fn adder(x: i32, y: i32) -> i32 {
    x + y
}

// Create a tool by just passing your function!
let tool = ToolBuilder::new("add")
    .description("Add two numbers")
    .parameters("x:i32:First number, y:i32:Second number")
    .ftool(adder)
    .build();
```

This demo shows multiple examples of the ultra-simple `ftool` API that automatically:
- Extracts parameters from JSON
- Calls your function with the right types
- Handles all the boilerplate for you

Available methods:
- `.ftool(fn)` - For 2 integer (i32) parameters
- `.ftool_f64(fn)` - For 2 float (f64) parameters
- `.ftool3_f64(fn)` - For 3 float (f64) parameters

**Run:**
```bash
cargo run --example tool_builder_demo
```

See the **[Tool Creation Guide](../docs/TOOL_CREATION_SIMPLE.md)** for complete documentation.

### Custom Tool (`custom_tool.rs`)

Create and use a custom tool by implementing the Tool trait:

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

### Forest of Agents (`forest_of_agents.rs`)

Create a collaborative multi-agent system where agents can communicate, delegate tasks, and share context:

```rust
use helios_engine::{Agent, Config, ForestBuilder};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    // Create a forest with specialized agents
    let mut forest = ForestBuilder::new()
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
- **Real-time streaming output**

### Forest of Agents with Coordinator (`forest_with_coordinator.rs`)

**NEW!** Enhanced Forest of Agents with coordinator-based planning system.

This advanced example demonstrates structured multi-agent collaboration with:

- **Coordinator-based planning**: Coordinator creates detailed task plans with dependencies
- **Shared task memory**: All agents read from and write to shared memory
- **Task dependencies**: Ensures proper execution order (e.g., research → analysis → writing → review)
- **Progress tracking**: Real-time monitoring of task completion
- **Three-phase workflow**:
  1. **Planning Phase**: Coordinator analyzes task and creates structured plan
  2. **Execution Phase**: Tasks executed in dependency order, agents update shared memory
  3. **Synthesis Phase**: Coordinator creates comprehensive final result

**Use Cases:**
- Content creation pipelines (Research → Draft → Edit → Review)
- Data analysis projects (Collection → Cleaning → Analysis → Report)
- Software development workflows (Requirements → Design → Implementation → Testing)
- Business strategy (Market Research → Analysis → Strategy → Review)

**Run:**
```bash
cargo run --example forest_with_coordinator
```

**Documentation:** See `docs/FOREST_COORDINATOR_PLANNING.md` for complete guide.

### Forest of Agents Simple Demo (`forest_simple_demo.rs`)

**NEW!** A simpler, more reliable demonstration of the planning system.

This example shows the coordinator-based planning with minimal complexity:

- **Simple Task**: Easy to understand task (listing benefits)
- **3 Agents**: Coordinator + 2 workers
- **Clear Output**: Shows plan creation and execution
- **Reliable**: Designed to work consistently with various LLMs

**Run:**
```bash
cargo run --example forest_simple_demo
```

**Best for:**
- Learning how the planning system works
- Testing your configuration
- Understanding task delegation flow

### Complete Demo (`complete_demo.rs`)

Showcase of all major features in one example.

## RAG Examples

### Agent with RAG (`agent_with_rag.rs`)

Create an agent with built-in RAG capabilities using the RAGTool:

```rust
use helios_engine::{Agent, Config, RAGTool, InMemoryVectorStore, OpenAIEmbeddings};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    // Create embeddings provider
    let embeddings = OpenAIEmbeddings::new(
        "https://api.openai.com/v1/embeddings".to_string(),
        std::env::var("OPENAI_API_KEY").unwrap(),
    );

    // Create vector store and RAG tool
    let vector_store = InMemoryVectorStore::new(embeddings);
    let rag_tool = RAGTool::new(vector_store);

    // Create agent with RAG capabilities
    let mut agent = Agent::builder("RAGAgent")
        .config(config)
        .system_prompt("You have access to a knowledge base. Use the rag_tool to retrieve relevant information.")
        .tool(Box::new(rag_tool))
        .build()
        .await?;

    // Add documents to the knowledge base
    agent.chat("Add this document about Rust: 'Rust is a systems programming language...'")
        .await?;

    // Query with semantic search
    let response = agent.chat("What is Rust programming?").await?;
    println!("Agent: {}", response);

    Ok(())
}
```

**Features:**
- **Integrated RAG agent** with semantic search capabilities
- **Document storage and retrieval** from conversation
- **Context-aware responses** using relevant knowledge
- **Easy-to-use interface** for RAG-powered agents

### RAG In-Memory (`rag_in_memory.rs`)

Simple RAG implementation using in-memory vector storage:

```rust
use helios_engine::{RAGSystem, InMemoryVectorStore, OpenAIEmbeddings, Document};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Create embeddings provider
    let embeddings = OpenAIEmbeddings::new(
        "https://api.openai.com/v1/embeddings".to_string(),
        std::env::var("OPENAI_API_KEY").unwrap(),
    );

    // Create RAG system with in-memory storage
    let vector_store = InMemoryVectorStore::new(embeddings);
    let mut rag_system = RAGSystem::new(vector_store);

    // Add documents
    let documents = vec![
        Document {
            id: "rust_doc".to_string(),
            content: "Rust is a systems programming language focused on safety and performance.".to_string(),
            metadata: std::collections::HashMap::new(),
        }
    ];

    rag_system.add_documents(documents).await?;

    // Search for relevant documents
    let results = rag_system.search("What is Rust?", 5).await?;
    for result in results {
        println!("Found: {} (score: {})", result.document.content, result.score);
    }

    Ok(())
}
```

**Features:**
- **In-memory vector storage** - no external dependencies
- **Simple document management** with metadata support
- **Semantic search** with relevance scoring
- **Fast retrieval** for small to medium document collections

### RAG Comparison (`rag_qdrant_comparison.rs`)

Compare performance between Qdrant and InMemory vector stores:

```rust
use helios_engine::{
    RAGSystem, InMemoryVectorStore, QdrantVectorStore,
    OpenAIEmbeddings, Document
};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let embeddings = OpenAIEmbeddings::new(
        "https://api.openai.com/v1/embeddings".to_string(),
        std::env::var("OPENAI_API_KEY").unwrap(),
    );

    // Test InMemory store
    println!("Testing InMemory Vector Store:");
    let in_memory_store = InMemoryVectorStore::new(embeddings.clone());
    let mut rag_memory = RAGSystem::new(in_memory_store);

    // Add test documents
    let documents = create_test_documents();
    rag_memory.add_documents(documents.clone()).await?;

    // Test Qdrant store (requires Qdrant running)
    println!("\nTesting Qdrant Vector Store:");
    let qdrant_store = QdrantVectorStore::new(
        "http://localhost:6333".to_string(),
        "test_collection".to_string(),
        embeddings,
    );
    let mut rag_qdrant = RAGSystem::new(qdrant_store);
    rag_qdrant.add_documents(documents).await?;

    // Compare search performance
    let query = "What are programming languages?";
    println!("\nComparing search results for: '{}'", query);

    let results_memory = rag_memory.search(query, 3).await?;
    let results_qdrant = rag_qdrant.search(query, 3).await?;

    println!("InMemory results: {}", results_memory.len());
    println!("Qdrant results: {}", results_qdrant.len());

    Ok(())
}
```

**Features:**
- **Performance comparison** between vector store implementations
- **Scalability testing** with different document sizes
- **Search quality analysis** across different backends
- **Benchmarking utilities** for RAG system evaluation

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
