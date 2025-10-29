# Helios Quick Reference

Quick reference for using Helios both as a CLI tool and as a library.

## CLI Usage

### Installation
```bash
cargo install helios
```

### Setup
```bash
helios init              # Create config.toml
# Edit config.toml with your API key
```

### Commands
```bash
helios                   # Interactive chat (default)
helios chat              # Interactive chat explicitly
helios ask "question"    # One-off question
helios --help            # Show help
```

### Options
```bash
-c, --config <FILE>      # Custom config file
-v, --verbose            # Verbose logging
-s, --system-prompt      # Custom system prompt
-m, --max-iterations     # Max tool iterations
```

### Interactive Commands
- `exit`, `quit` - Exit chat
- `clear` - Clear history
- `tools` - List tools
- `help` - Show help

## Library Usage

### Basic Setup
```rust
use helios::{LLMClient, ChatMessage};
use helios::config::LLMConfig;

let config = LLMConfig {
    model_name: "gpt-3.5-turbo".to_string(),
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: std::env::var("OPENAI_API_KEY").unwrap(),
    temperature: 0.7,
    max_tokens: 2048,
};
```

### Simple Call
```rust
let client = LLMClient::new(config);
let messages = vec![ChatMessage::user("Hello!")];
let response = client.chat(messages, None).await?;
```

### Conversation
```rust
let mut session = ChatSession::new()
    .with_system_prompt("You are helpful.");

session.add_user_message("Hello");
let resp = client.chat(session.get_messages(), None).await?;
session.add_assistant_message(&resp.content);
```

### Agent with Tools
```rust
let config = Config::from_file("config.toml")?;
let mut agent = Agent::builder("MyAgent")
    .config(config)
    .system_prompt("You are helpful.")
    .tool(Box::new(CalculatorTool))
    .build()?;

let response = agent.chat("Calculate 2+2").await?;
```

## Configuration

### File (config.toml)
```toml
[llm]
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-key"
temperature = 0.7
max_tokens = 2048
```

### Providers

**OpenAI:**
```toml
base_url = "https://api.openai.com/v1"
```

**Local (LM Studio):**
```toml
base_url = "http://localhost:1234/v1"
```

**Ollama:**
```toml
base_url = "http://localhost:11434/v1"
```

## Common Patterns

### Environment Variables
```rust
api_key: std::env::var("OPENAI_API_KEY").unwrap()
```

### Error Handling
```rust
match client.chat(messages, None).await {
    Ok(resp) => println!("{}", resp.content),
    Err(e) => eprintln!("Error: {}", e),
}
```

## More Information

- [Full Documentation](README.md)
- [Detailed Usage](USAGE.md)
- [Library Guide](docs/USING_AS_CRATE.md)
- [Publishing](PUBLISHING.md)
