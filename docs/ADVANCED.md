# Advanced Features Guide

This guide covers Helios Engine's advanced features including multi-agent systems, conversation management, streaming responses, and specialized capabilities.

## 🆕 Forest of Agents

Create collaborative multi-agent systems where agents can communicate, delegate tasks, and share context. The Forest system enables complex workflows with specialized agents working together.

### Basic Forest Setup

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
        .agent(
            "writer".to_string(),
            Agent::builder("writer")
                .system_prompt("You create content and documentation.")
        )
        .build()
        .await?;
```

### Inter-Agent Communication

#### Direct Messaging
Send messages between specific agents:

```rust
// Send a direct message from coordinator to researcher
forest
    .send_message(
        &"coordinator".to_string(),
        Some(&"researcher".to_string()),
        "Please research the latest findings on sustainable energy.".to_string(),
    )
    .await?;
```

#### Broadcasting
Send messages to all agents:

```rust
// Broadcast to all agents
forest
    .send_message(
        &"coordinator".to_string(),
        None, // None means broadcast to all
        "Team meeting in 5 minutes!".to_string(),
    )
    .await?;
```

### Collaborative Task Execution

Execute complex tasks that require multiple agents working together:

```rust
let result = forest
    .execute_collaborative_task(
        &"coordinator".to_string(),
        "Create a comprehensive guide on sustainable practices".to_string(),
        vec!["researcher".to_string(), "writer".to_string()],
    )
    .await?;

println!("Collaborative result: {}", result);
```

### Forest Architecture

```
┌─────────────────────────────────────┐
│           Forest Builder            │
├─────────────────────────────────────┤
│  • agent(name, agent)               │
│  • config(config)                   │
│  • build() -> Forest                │
└─────────────────┬───────────────────┘
                  │
         ┌────────▼────────┐
         │     Forest      │
         ├─────────────────┤
         │  • agents       │
         │  • send_message │
         │  • execute_collab │
         └────────┬────────┘
                  │
        ┌─────────┴─────────┐
        │                   │
┌───────▼──────┐   ┌────────▼────────┐
│   Agent A    │   │    Agent B      │
│              │   │                 │
│ • personality│   │ • personality   │
│ • tools      │   │ • tools         │
│ • memory     │   │ • memory        │
└──────────────┘   └─────────────────┘
```

## Conversation Management

### Session Memory & Metadata

Track agent state and conversation metadata across interactions:

```rust
let mut agent = Agent::builder("Assistant")
    .config(config)
    .build()
    .await?;

// Set agent memory (namespaced under "agent:" prefix)
agent.set_memory("user_preference", "concise");
agent.set_memory("tasks_completed", "0");

// Get memory values
if let Some(pref) = agent.get_memory("user_preference") {
    println!("User prefers: {}", pref);
}

// Increment counters
agent.increment_tasks_completed();
agent.increment_counter("files_processed");

// Get session summary
println!("{}", agent.get_session_summary());

// Clear only agent memory (preserves general session metadata)
agent.clear_memory();
```

### ChatSession Management

The `ChatSession` provides low-level conversation management:

```rust
use helios_engine::ChatSession;

let mut session = ChatSession::new();

// Set general session metadata
session.set_metadata("session_id", "abc123");
session.set_metadata("start_time", chrono::Utc::now().to_rfc3339());

// Retrieve metadata
if let Some(id) = session.get_metadata("session_id") {
    println!("Session ID: {}", id);
}

// Get session summary
println!("{}", session.get_summary());
```

## Advanced Agent Patterns

### Tool Chaining

Agents automatically chain tool calls to solve complex problems:

```rust
// The agent can use multiple tools in sequence automatically
let response = agent.chat(
    "Calculate 10 * 5, then search for files containing that result"
).await?;
```

### Custom System Prompts

Create specialized agents with custom personalities and capabilities:

```rust
let mut agent = Agent::builder("CodeReviewer")
    .config(config)
    .system_prompt(r#"
    You are an expert code reviewer with deep knowledge of:
    - Rust programming language
    - Software architecture principles
    - Security best practices
    - Performance optimization

    When reviewing code, provide:
    1. Overall assessment
    2. Specific issues with severity levels
    3. Suggested improvements
    4. Security considerations
    "#)
    .tool(Box::new(FileReadTool))
    .tool(Box::new(TextProcessorTool))
    .build()
    .await?;
```

### Memory-Augmented Agents

Combine memory tools with conversation context:

```rust
use helios_engine::MemoryDBTool;

let mut agent = Agent::builder("LearningAgent")
    .config(config)
    .system_prompt("You learn from conversations and remember important information.")
    .tool(Box::new(MemoryDBTool::new()))
    .build()
    .await?;

// Agent can now store and retrieve information across conversations
agent.chat("Remember that my favorite programming language is Rust").await?;
agent.chat("What is my favorite programming language?").await?; // Remembers "Rust"
```

## Streaming and Real-Time Features

### Streaming Responses

Enable real-time token streaming for immediate responses:

```rust
use helios_engine::LLMClient;
use futures::stream::StreamExt;

let client = LLMClient::new(provider).await?;
let messages = vec![/* your messages */];

// Stream the response
let mut stream = client.chat_stream(messages, None).await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(response) => print!("{}", response.content),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

See **[Streaming Guide](STREAMING.md)** for detailed streaming documentation.

## Custom LLM Providers

### Implementing Custom Providers

Extend Helios with custom LLM backends by implementing the `LLMProvider` trait:

```rust
use async_trait::async_trait;
use helios_engine::{LLMProvider, LLMRequest, LLMResponse};

struct CustomProvider;

#[async_trait]
impl LLMProvider for CustomProvider {
    async fn generate(&self, request: LLMRequest) -> helios_engine::Result<LLMResponse> {
        // Your custom implementation
        // - Format the request for your API
        // - Make HTTP call
        // - Parse response
        // - Return LLMResponse

        todo!()
    }
}
```

### Provider Configuration

Use custom providers with the dual LLMProviderType system:

```rust
use helios_engine::{LLMClient, llm::LLMProviderType};

// Create client with custom provider
let client = LLMClient::new(LLMProviderType::Custom(Box::new(CustomProvider))).await?;
```

## Performance Optimization

### Connection Pooling

For high-throughput applications, configure connection pooling:

```rust
use reqwest::Client;

let http_client = Client::builder()
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(std::time::Duration::from_secs(30))
    .build()?;

let config = LLMConfig {
    // ... other config
    client: Some(http_client),
    // ...
};
```

### Memory Management

For memory-constrained environments:

```rust
// Limit conversation history
let mut agent = Agent::builder("LightAgent")
    .config(config)
    .max_history_length(50) // Keep only last 50 messages
    .build()
    .await?;

// Periodic cleanup
agent.clear_old_history(24 * 60 * 60); // Clear messages older than 24 hours
```

### Concurrent Processing

Handle multiple conversations concurrently:

```rust
use tokio::task;
use std::sync::Arc;

let agent = Arc::new(agent);

let handles: Vec<_> = (0..10).map(|i| {
    let agent = Arc::clone(&agent);
    task::spawn(async move {
        let response = agent.chat(format!("Hello from task {}", i)).await?;
        Ok::<_, helios_engine::Error>(response)
    })
}).collect();

for handle in handles {
    let result = handle.await??;
    println!("Result: {}", result);
}
```

## Error Handling and Resilience

### Graceful Degradation

Configure agents to handle provider failures:

```rust
// Auto mode: tries local first, falls back to remote
let config = Config::from_file_with_mode("config.toml", LLMMode::Auto).await?;

// Manual fallback logic
async fn chat_with_fallback(
    agent: &mut Agent,
    message: &str,
) -> helios_engine::Result<String> {
    match agent.chat(message).await {
        Ok(response) => Ok(response),
        Err(e) => {
            eprintln!("Primary provider failed: {}", e);
            // Try with fallback configuration
            agent.switch_to_fallback_provider().await?;
            agent.chat(message).await
        }
    }
}
```

### Retry Logic

Implement custom retry strategies:

```rust
use std::time::Duration;

async fn chat_with_retry(
    agent: &mut Agent,
    message: &str,
    max_retries: u32,
) -> helios_engine::Result<String> {
    let mut attempt = 0;
    loop {
        match agent.chat(message).await {
            Ok(response) => return Ok(response),
            Err(e) if attempt < max_retries => {
                attempt += 1;
                tokio::time::sleep(Duration::from_millis(500 * attempt as u64)).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Monitoring and Observability

### Logging Integration

Enable detailed logging for debugging:

```rust
use tracing_subscriber;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Your agent code here
    // Logs will include detailed information about LLM calls, tool usage, etc.
}
```

### Metrics Collection

Track agent performance and usage:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct MetricsAgent {
    agent: Agent,
    requests_total: AtomicU64,
    tool_calls_total: AtomicU64,
}

impl MetricsAgent {
    async fn chat(&mut self, message: impl Into<String>) -> helios_engine::Result<String> {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        let response = self.agent.chat(message).await?;
        Ok(response)
    }

    fn get_metrics(&self) -> (u64, u64) {
        (
            self.requests_total.load(Ordering::Relaxed),
            self.tool_calls_total.load(Ordering::Relaxed),
        )
    }
}
```

## Security Considerations

### Input Validation

Always validate inputs to prevent injection attacks:

```rust
// For file operations
fn validate_file_path(path: &str) -> helios_engine::Result<()> {
    if path.contains("..") || path.starts_with('/') {
        return Err(helios_engine::Error::InvalidParameter(
            "Invalid file path".to_string()
        ));
    }
    Ok(())
}

// For shell commands
fn validate_command(cmd: &str) -> helios_engine::Result<()> {
    let forbidden = ["rm", "del", "format", "sudo"];
    for word in forbidden {
        if cmd.contains(word) {
            return Err(helios_engine::Error::InvalidParameter(
                format!("Forbidden command: {}", word)
            ));
        }
    }
    Ok(())
}
```

### API Key Management

Securely manage API keys:

```rust
use std::env;

// Environment variables (recommended)
let api_key = std::env::var("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY must be set");

// Configuration files
let config = Config::from_file("secure_config.toml")?;

// Never hardcode keys
// ❌ BAD: let api_key = "sk-1234567890abcdef";
// ✅ GOOD: Load from environment or secure config
```

## Next Steps

- **[RAG Guide](RAG.md)** - Retrieval-Augmented Generation
- **[Streaming Guide](STREAMING.md)** - Real-time responses
- **[API Reference](API.md)** - Complete technical reference
- **[Examples](../examples/)** - Advanced usage examples
