# Using Helios as a Crate

This guide shows you how to use Helios as a library crate in your Rust projects to call LLM models directly.

## Installation

Add Helios to your `Cargo.toml`:

```toml
[dependencies]
helios-engine = { path = "../helios-engine" }  # If using locally
# or
helios-engine = "0.3.7"  # From crates.io
tokio = { version = "1.35", features = ["full"] }
```

## Quick Start: Direct LLM Calls

### 1. Basic LLM Client Usage

The simplest way to call models directly without the Agent abstraction:

```rust
use helios_engine::{LLMClient, LLMProvider, ChatMessage};
use helios_engine::config::LLMConfig;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Create LLM configuration
    let llm_config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: "sk-your-api-key-here".to_string(),
        temperature: 0.7,
        max_tokens: 2048,
    };

    // Create LLM client
    let client = LLMClient::new(helios_engine::llm::LLMProviderType::Remote(llm_config)).await?;

    // Prepare messages
    let messages = vec![
        ChatMessage::system("You are a helpful assistant."),
        ChatMessage::user("What is the capital of France?"),
    ];

    // Call the model
    let response = client.chat(messages, None, None, None, None).await?;
    println!("Response: {}", response.content);

    Ok(())
}
```

### 2. Using Configuration File

Load configuration from a TOML file:

```rust
use helios_engine::{Config, LLMClient, LLMProvider, ChatMessage};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Load from config file
    let config = Config::from_file("config.toml")?;
    
    // Create client from config
    let client = LLMClient::new(helios_engine::llm::LLMProviderType::Remote(config.llm)).await?;
    
    // Make a call
    let messages = vec![
        ChatMessage::user("Hello, how are you?"),
    ];
    
    let response = client.chat(messages, None, None, None, None).await?;
    println!("{}", response.content);
    
    Ok(())
}
```

### 3. Conversation with Message History

Building a conversation with context:

```rust
use helios_engine::{LLMClient, ChatMessage, ChatSession};
use helios_engine::config::LLMConfig;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let llm_config = LLMConfig {
        model_name: "gpt-4".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY").unwrap(),
        temperature: 0.7,
        max_tokens: 2048,
    };

    let client = LLMClient::new(helios_engine::llm::LLMProviderType::Remote(llm_config)).await?;
    
    // Use ChatSession to manage conversation
    let mut session = ChatSession::new()
        .with_system_prompt("You are a helpful math tutor.");
    
    // First message
    session.add_user_message("I need help with algebra.");
    let response1 = client.chat(session.get_messages(), None, None, None, None).await?;
    session.add_assistant_message(&response1.content);
    println!("Assistant: {}", response1.content);
    
    // Follow-up message (with context)
    session.add_user_message("Can you explain quadratic equations?");
    let response2 = client.chat(session.get_messages(), None, None, None, None).await?;
    session.add_assistant_message(&response2.content);
    println!("Assistant: {}", response2.content);
    
    Ok(())
}
```

### 4. Low-Level API Access

For maximum control, use the `generate` method directly:

```rust
use helios_engine::{LLMClient, LLMProvider, ChatMessage};
use helios_engine::llm::LLMRequest;
use helios_engine::config::LLMConfig;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let llm_config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: "your-key".to_string(),
        temperature: 0.7,
        max_tokens: 2048,
    };
    
    let client = LLMClient::new(helios_engine::llm::LLMProviderType::Remote(llm_config)).await?;

    // Build custom request
    let request = LLMRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![
            ChatMessage::user("Tell me a joke"),
        ],
        temperature: Some(0.9),
        max_tokens: Some(100),
        tools: None,
        tool_choice: None,
        stream: None,
        stop: None,
    };

    // Get full response with metadata
    let response = client.generate(request).await?;
    
    println!("Model: {}", response.model);
    println!("Tokens used: {}", response.usage.total_tokens);
    println!("Response: {}", response.choices[0].message.content);
    
    Ok(())
}
```

## Different LLM Providers

### OpenAI

```rust
let llm_config = LLMConfig {
    model_name: "gpt-4".to_string(),
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: std::env::var("OPENAI_API_KEY").unwrap(),
    temperature: 0.7,
    max_tokens: 2048,
};
```

### Local Models (LM Studio, Ollama, etc.)

```rust
// LM Studio
let llm_config = LLMConfig {
    model_name: "local-model".to_string(),
    base_url: "http://localhost:1234/v1".to_string(),
    api_key: "not-needed".to_string(),
    temperature: 0.7,
    max_tokens: 2048,
};

// Ollama
let llm_config = LLMConfig {
    model_name: "llama2".to_string(),
    base_url: "http://localhost:11434/v1".to_string(),
    api_key: "not-needed".to_string(),
    temperature: 0.7,
    max_tokens: 2048,
};
```

### Azure OpenAI

```rust
let llm_config = LLMConfig {
    model_name: "gpt-35-turbo".to_string(),
    base_url: "https://your-resource.openai.azure.com/openai/deployments/your-deployment".to_string(),
    api_key: std::env::var("AZURE_OPENAI_KEY").unwrap(),
    temperature: 0.7,
    max_tokens: 2048,
};
```

## Advanced Examples

### Interactive CLI Chat

```rust
use helios_engine::{LLMClient, ChatSession};
use helios_engine::config::LLMConfig;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let llm_config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY").unwrap(),
        temperature: 0.7,
        max_tokens: 2048,
    };
    
    let client = LLMClient::new(helios_engine::llm::LLMProviderType::Remote(llm_config)).await?;

    let mut session = ChatSession::new()
        .with_system_prompt("You are a helpful assistant.");

    println!("Chat started! Type 'exit' to quit.\n");

    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "exit" {
            println!("Goodbye!");
            break;
        }

        session.add_user_message(input);
        
        match client.chat(session.get_messages(), None, None, None, None).await {
            Ok(response) => {
                session.add_assistant_message(&response.content);
                println!("Assistant: {}\n", response.content);
            }
            Err(e) => {
                eprintln!("Error: {}\n", e);
            }
        }
    }

    Ok(())
}
```

### Batch Processing

```rust
use helios_engine::{LLMClient, ChatMessage};
use helios_engine::config::LLMConfig;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let client = LLMClient::new(LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY").unwrap(),
        temperature: 0.7,
        max_tokens: 2048,
    });

    let prompts = vec![
        "What is 2+2?",
        "What is the capital of France?",
        "Who wrote Hamlet?",
    ];

    for prompt in prompts {
        let messages = vec![ChatMessage::user(prompt)];
        let response = client.chat(messages, None).await?;
        println!("Q: {}", prompt);
        println!("A: {}\n", response.content);
    }

    Ok(())
}
```

### Parallel Requests

```rust
use helios_engine::{LLMClient, ChatMessage};
use helios_engine::config::LLMConfig;
use tokio::task;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let llm_config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY").unwrap(),
        temperature: 0.7,
        max_tokens: 2048,
    };

    let prompts = vec![
        "What is 2+2?",
        "What is the capital of France?",
        "Who wrote Hamlet?",
    ];

    let mut handles = vec![];

    for prompt in prompts {
        let client = LLMClient::new(llm_config.clone());
        let handle = task::spawn(async move {
            let messages = vec![ChatMessage::user(prompt)];
            client.chat(messages, None).await
        });
        handles.push((prompt, handle));
    }

    for (prompt, handle) in handles {
        match handle.await {
            Ok(Ok(response)) => {
                println!("Q: {}", prompt);
                println!("A: {}\n", response.content);
            }
            Ok(Err(e)) => eprintln!("Error for '{}': {}", prompt, e),
            Err(e) => eprintln!("Task error: {}", e),
        }
    }

    Ok(())
}
```

## API Reference

### Core Types

- `LLMClient`: The main client for making LLM calls
- `LLMConfig`: Configuration for the LLM (model, API key, etc.)
- `ChatMessage`: Represents a message in a conversation
- `ChatSession`: Manages conversation state and message history
- `Role`: Message role (System, User, Assistant, Tool)

### Key Methods

#### LLMClient

```rust
impl LLMClient {
    // Create a new client with configuration
    pub fn new(config: LLMConfig) -> Self

    // Simple chat interface (recommended)
    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<ChatMessage>

    // Low-level interface with full control
    pub async fn generate(&self, request: LLMRequest) -> Result<LLMResponse>
}
```

#### ChatMessage

```rust
impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self
    pub fn user(content: impl Into<String>) -> Self
    pub fn assistant(content: impl Into<String>) -> Self
    pub fn tool(content: impl Into<String>, tool_call_id: impl Into<String>) -> Self
}
```

#### ChatSession

```rust
impl ChatSession {
    pub fn new() -> Self
    pub fn with_system_prompt(self, prompt: impl Into<String>) -> Self
    pub fn add_message(&mut self, message: ChatMessage)
    pub fn add_user_message(&mut self, content: impl Into<String>)
    pub fn add_assistant_message(&mut self, content: impl Into<String>)
    pub fn get_messages(&self) -> Vec<ChatMessage>
    pub fn clear(&mut self)
}
```

## Error Handling

Helios uses the `Result<T>` type with `HeliosError`:

```rust
use helios_engine::{LLMClient, ChatMessage, HeliosError};

match client.chat(messages, None).await {
    Ok(response) => {
        println!("Success: {}", response.content);
    }
    Err(HeliosError::LLMError(msg)) => {
        eprintln!("LLM error: {}", msg);
    }
    Err(HeliosError::ConfigError(msg)) => {
        eprintln!("Config error: {}", msg);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## Best Practices

1. **Use environment variables for API keys**:
   ```rust
   api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default()
   ```

2. **Manage conversation context with ChatSession**:
   ```rust
   let mut session = ChatSession::new().with_system_prompt("...");
   ```

3. **Handle errors gracefully**:
   ```rust
   if let Err(e) = client.chat(messages, None).await {
       eprintln!("Request failed: {}", e);
   }
   ```

4. **Clone configuration when needed**:
   ```rust
   let config = llm_config.clone();
   ```

5. **Use appropriate temperature settings**:
   - `0.0-0.3`: Deterministic, factual responses
   - `0.7-0.9`: Creative, varied responses
   - `1.0+`: Very random (usually not recommended)

## Complete Example Project

Create a new project:

```bash
cargo new my_llm_app
cd my_llm_app
```

Update `Cargo.toml`:

```toml
[package]
name = "my_llm_app"
version = "0.1.0"
edition = "2021"

[dependencies]
helios-engine = { path = "../helios-engine" }
tokio = { version = "1.35", features = ["full"] }
```

Create `src/main.rs`:

```rust
use helios_engine::{LLMClient, ChatSession};
use helios_engine::config::LLMConfig;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Setup
    let client = LLMClient::new(LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY")
            .expect("OPENAI_API_KEY not set"),
        temperature: 0.7,
        max_tokens: 2048,
    });

    let mut session = ChatSession::new()
        .with_system_prompt("You are a helpful AI assistant.");

    println!("AI Chat (type 'exit' to quit)\n");

    // Chat loop
    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "exit" {
            println!("Goodbye! 👋");
            break;
        }

        session.add_user_message(input);

        match client.chat(session.get_messages(), None).await {
            Ok(response) => {
                session.add_assistant_message(&response.content);
                println!("\nAI: {}\n", response.content);
            }
            Err(e) => {
                eprintln!("\nError: {}\n", e);
            }
        }
    }

    Ok(())
}
```

Run it:

```bash
export OPENAI_API_KEY="your-key-here"
cargo run
```

## Next Steps

- Explore the [Agent API](./QUICKSTART.md) for higher-level abstractions
- Learn about [custom tools](../README.md#creating-custom-tools)
- Check out more [examples](../examples/)
- Read the [full API documentation](./API.md)

## Support

- [Full Documentation](../README.md)
- [Examples](../examples/)
- [Issue Tracker](https://github.com/Ammar-Alnagar/Helios-Engine/issues)
