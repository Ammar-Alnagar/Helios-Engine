# Helios Usage Guide

This guide covers all the ways to use Helios - both as a CLI tool and as a library crate.

## Table of Contents

1. [Installation](#installation)
2. [CLI Usage](#cli-usage)
3. [Library Usage](#library-usage)
4. [Configuration](#configuration)
5. [Examples](#examples)

## Installation

### Install as CLI Tool

Once published to crates.io, install globally:

```bash
cargo install helios-engine
```

Then use anywhere:

```bash
helios-engine --help
```

### Use as Library

Add to your project's `Cargo.toml`:

```toml
[dependencies]
helios-engine = "0.3.7"
tokio = { version = "1.35", features = ["full"] }
```

### Build from Source

```bash
git clone https://github.com/yourusername/helios.git
cd helios

# Build and install CLI
cargo install --path .

# Or just build
cargo build --release
./target/release/helios
```

## CLI Usage

### Initialize Configuration

Create a new config file:

```bash
helios-engine init
```

This creates `config.toml` with default settings. Edit it to add your API key.

Create config in a custom location:

```bash
helios-engine init --output ~/.helios/config.toml
```

### Interactive Chat

Start an interactive chat session (default command):

```bash
helios-engine
# or explicitly
helios-engine chat
```

With custom config file:

```bash
helios-engine --config /path/to/config.toml chat
```

With custom system prompt:

```bash
helios-engine chat --system-prompt "You are a Python expert"
```

With custom max iterations:

```bash
helios-engine chat --max-iterations 10
```

### One-Off Questions

Ask a single question without interactive mode:

```bash
helios-engine ask "What is the capital of France?"
```

With custom config:

```bash
helios-engine --config my-config.toml ask "Calculate 123 * 456"
```

### Verbose Mode

Enable debug logging:

```bash
helios-engine --verbose chat
```

### CLI Examples

```bash
# Initialize config
helios-engine init

# Start chat with default settings
helios-engine

# Chat with custom prompt
helios-engine chat -s "You are a Rust expert"

# Single question
helios-engine ask "What is 2+2?"

# Verbose logging
helios -v chat

# Custom config location
helios -c ~/.config/helios.toml chat
```

### CLI Help

```bash
# General help
helios-engine --help

# Help for specific command
helios-engine chat --help
helios-engine init --help
helios-engine ask --help
```

## Library Usage

### Use Case 1: Direct LLM Calls (Simplest)

For simple, direct calls to LLM models:

```rust
use helios_engine::{LLMClient, ChatMessage, LLMConfig};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Configure LLM
    let config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY").unwrap(),
        temperature: 0.7,
        max_tokens: 2048,
    };

    // Create client
    let client = LLMClient::new(config);

    // Make a call
    let messages = vec![
        ChatMessage::system("You are a helpful assistant."),
        ChatMessage::user("What is Rust?"),
    ];

    let response = client.chat(messages, None).await?;
    println!("{}", response.content);

    Ok(())
}
```

### Use Case 2: Conversation with Context

Manage multi-turn conversations:

```rust
use helios_engine::{LLMClient, ChatSession, LLMConfig};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY").unwrap(),
        temperature: 0.7,
        max_tokens: 2048,
    };

    let client = LLMClient::new(config);
    
    // Create session with system prompt
    let mut session = ChatSession::new()
        .with_system_prompt("You are a helpful assistant.");

    // First message
    session.add_user_message("What is Rust?");
    let response = client.chat(session.get_messages(), None).await?;
    session.add_assistant_message(&response.content);
    println!("Assistant: {}", response.content);

    // Follow-up (with context)
    session.add_user_message("What are its main features?");
    let response = client.chat(session.get_messages(), None).await?;
    session.add_assistant_message(&response.content);
    println!("Assistant: {}", response.content);

    Ok(())
}
```

### Use Case 3: Agent with Tools

For advanced use cases with tools:

```rust
use helios_engine::{Agent, Config, CalculatorTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Load config from file
    let config = Config::from_file("config.toml")?;

    // Create agent with tools
    let mut agent = Agent::builder("MyAgent")
        .config(config)
        .system_prompt("You are a helpful assistant with tools.")
        .tool(Box::new(CalculatorTool))
        .max_iterations(5)
        .build()
    .await?;

    // Chat
    let response = agent.chat("What is 123 * 456?").await?;
    println!("{}", response);

    Ok(())
}
```

### Use Case 4: Custom Tool Implementation

Create your own tools:

```rust
use helios_engine::{Tool, ToolParameter, ToolResult, Agent, Config};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "get_weather"
    }

    fn description(&self) -> &str {
        "Get weather for a location"
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
        let location = args["location"].as_str()
            .unwrap_or("Unknown");

        // Simulate weather API call
        let weather = format!("Sunny, 72°F in {}", location);
        Ok(ToolResult::success(weather))
    }
}

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    let mut agent = Agent::builder("WeatherAgent")
        .config(config)
        .system_prompt("You help with weather info.")
        .tool(Box::new(WeatherTool))
        .build()
    .await?;

    let response = agent.chat("What's the weather in Paris?").await?;
    println!("{}", response);

    Ok(())
}
```

### Use Case 5: Multiple Agents

Create and manage multiple agents:

```rust
use helios_engine::{Agent, Config};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    let mut math_agent = Agent::builder("MathExpert")
        .config(config.clone())
        .system_prompt("You are a math expert.")
        .build()
    .await?;

    let mut writer_agent = Agent::builder("Writer")
        .config(config)
        .system_prompt("You are a creative writer.")
        .build()
    .await?;

    let math_response = math_agent.chat("What is 15 * 23?").await?;
    println!("Math Expert: {}", math_response);

    let writer_response = writer_agent.chat("Write a haiku about code").await?;
    println!("Writer: {}", writer_response);

    Ok(())
}
```

## Configuration

### Configuration File Format

Create a `config.toml` file:

```toml
[llm]
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-api-key-here"
temperature = 0.7
max_tokens = 2048
```

### Configuration in Code

Create config programmatically:

```rust
use helios_engine::config::LLMConfig;

let config = LLMConfig {
    model_name: "gpt-3.5-turbo".to_string(),
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: std::env::var("OPENAI_API_KEY").unwrap(),
    temperature: 0.7,
    max_tokens: 2048,
};
```

### Provider-Specific Configurations

**OpenAI:**
```toml
[llm]
model_name = "gpt-4"
base_url = "https://api.openai.com/v1"
api_key = "sk-..."
```

**Azure OpenAI:**
```toml
[llm]
model_name = "gpt-35-turbo"
base_url = "https://your-resource.openai.azure.com/openai/deployments/your-deployment"
api_key = "your-azure-key"
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

### Environment Variables

Use environment variables for sensitive data:

```rust
use helios_engine::config::LLMConfig;

let config = LLMConfig {
    model_name: "gpt-3.5-turbo".to_string(),
    base_url: std::env::var("LLM_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
    api_key: std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY must be set"),
    temperature: 0.7,
    max_tokens: 2048,
};
```

## Examples

### CLI Examples

#### Quick Setup
```bash
# Initialize and configure
helios-engine init
# Edit config.toml with your API key
nano config.toml
# Start chatting
helios-engine
```

#### One-Off Tasks
```bash
# Quick question
helios-engine ask "Explain async/await in Rust"

# With calculation
helios-engine ask "What is 15% of 230?"
```

#### Custom Sessions
```bash
# Code review session
helios-engine chat -s "You are a code reviewer. Be thorough and constructive."

# Math tutor session
helios-engine chat -s "You are a patient math tutor for beginners."
```

### Library Examples

#### Simple CLI App
```rust
use helios_engine::{LLMClient, ChatMessage, LLMConfig};
use std::env;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <question>", args[0]);
        return Ok(());
    }

    let question = args[1..].join(" ");

    let config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: env::var("OPENAI_API_KEY")?,
        temperature: 0.7,
        max_tokens: 2048,
    };

    let client = LLMClient::new(config);
    let messages = vec![ChatMessage::user(&question)];
    let response = client.chat(messages, None).await?;
    
    println!("{}", response.content);
    Ok(())
}
```

#### Web Service Integration
```rust
use helios_engine::{LLMClient, ChatMessage, LLMConfig};
use std::sync::Arc;

// In your web service
struct AppState {
    llm_client: Arc<LLMClient>,
}

async fn handle_request(
    state: Arc<AppState>,
    user_message: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let messages = vec![
        ChatMessage::system("You are a helpful API assistant."),
        ChatMessage::user(user_message),
    ];

    let response = state.llm_client.chat(messages, None).await?;
    Ok(response.content)
}

#[tokio::main]
async fn main() {
    let config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY").unwrap(),
        temperature: 0.7,
        max_tokens: 2048,
    };

    let state = Arc::new(AppState {
        llm_client: Arc::new(LLMClient::new(config)),
    });

    // Use in your web framework...
}
```

## Best Practices

### Security
- Never commit API keys to version control
- Use environment variables for sensitive data
- Rotate API keys regularly

### Performance
- Reuse `LLMClient` instances (they're cheap to clone)
- Use appropriate `max_tokens` to control costs
- Consider caching responses for common queries

### Error Handling
```rust
match client.chat(messages, None).await {
    Ok(response) => println!("{}", response.content),
    Err(helios_engine::HeliosError::LLMError(e)) => {
        eprintln!("LLM error: {}", e);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

### Temperature Settings
- **0.0-0.3**: Deterministic, factual responses
- **0.7**: Balanced creativity and coherence (default)
- **0.9-1.0**: Very creative, less predictable

## Troubleshooting

### CLI Issues

**"Configuration file not found"**
```bash
helios-engine init
# Edit config.toml
helios-engine chat
```

**"API key not configured"**
- Edit config.toml and add your API key
- Or use environment variable: `export OPENAI_API_KEY="sk-..."`

### Library Issues

**"Cannot find module helios"**
- Add to Cargo.toml: `helios-engine = "0.3.7"`
- Run: `cargo build`

**Connection errors**
- Check internet connectivity
- Verify base_url is correct
- Ensure API key is valid

## Additional Resources

- [Full Documentation](README.md)
- [API Reference](docs/API.md)
- [Using as Crate Guide](docs/USING_AS_CRATE.md)
- [Examples Directory](examples/)
- [Publishing Guide](PUBLISHING.md)
