# Streaming Support in Helios Engine

Helios Engine supports streaming responses from LLMs, providing real-time output as the model generates text. This guide explains how to use streaming in both CLI and library modes.

## Overview

Streaming provides several benefits:
- **Real-time feedback**: See responses as they're generated
- **Better UX**: No waiting for complete responses
- **Thinking tags**: See reasoning process from models that support it
- **Progress indication**: Know the model is working

## CLI Usage

### Interactive Chat (Streaming by Default)

```bash
helios-engine chat
```

The interactive chat mode uses streaming automatically. You'll see:
- Text appearing character by character
- Thinking tags displayed when the model uses them (e.g., o1 models)
- Real-time progress

### One-off Questions

```bash
helios-engine ask "What is Rust?"
```

The `ask` command also uses streaming for immediate feedback.

### Thinking Tags

When using models that support thinking (like OpenAI's o1 series), you'll see:

```
You: Solve this complex problem...

🤖: 
💭 [Thinking...........]
Let me break this down step by step...
```

The `💭 [Thinking...]` indicator shows the model is reasoning about the problem.

## Library Usage

### Basic Streaming

```rust
use helios_engine::{LLMClient, ChatMessage};
use helios_engine::config::LLMConfig;
use std::io::{self, Write};

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
    let messages = vec![
        ChatMessage::user("Tell me about Rust"),
    ];

    // Stream with callback
    let response = client.chat_stream(messages, None, |chunk| {
        print!("{}", chunk);
        io::stdout().flush().unwrap();
    }).await?;

    println!("\n");
    
    Ok(())
}
```

### Streaming with Conversation

```rust
use helios_engine::{LLMClient, ChatSession};
use helios_engine::config::LLMConfig;
use std::io::{self, Write};

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
    let mut session = ChatSession::new()
        .with_system_prompt("You are a helpful assistant.");

    // First message
    session.add_user_message("What is 2+2?");
    
    print!("Assistant: ");
    io::stdout().flush()?;
    
    let response = client.chat_stream(session.get_messages(), None, |chunk| {
        print!("{}", chunk);
        io::stdout().flush().unwrap();
    }).await?;
    
    session.add_assistant_message(&response.content);
    println!("\n");

    // Follow-up message (with context)
    session.add_user_message("Now multiply that by 3");
    
    print!("Assistant: ");
    io::stdout().flush()?;
    
    let response = client.chat_stream(session.get_messages(), None, |chunk| {
        print!("{}", chunk);
        io::stdout().flush().unwrap();
    }).await?;
    
    session.add_assistant_message(&response.content);
    println!("\n");

    Ok(())
}
```

### Detecting Thinking Tags

For models that output thinking tags (like `<thinking>...</thinking>`):

```rust
use helios_engine::{LLMClient, ChatMessage};
use helios_engine::config::LLMConfig;
use std::io::{self, Write};

struct ThinkingTracker {
    in_thinking: bool,
    thinking_buffer: String,
}

impl ThinkingTracker {
    fn new() -> Self {
        Self {
            in_thinking: false,
            thinking_buffer: String::new(),
        }
    }

    fn process_chunk(&mut self, chunk: &str) -> Option<String> {
        let mut output = String::new();
        let mut chars = chunk.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '<' {
                let remaining: String = chars.clone().collect();
                if remaining.starts_with("thinking>") {
                    self.in_thinking = true;
                    self.thinking_buffer.clear();
                    output.push_str("\n💭 [Thinking");
                    for _ in 0..9 { chars.next(); }
                    continue;
                } else if remaining.starts_with("/thinking>") {
                    self.in_thinking = false;
                    output.push_str("]\n");
                    for _ in 0..10 { chars.next(); }
                    continue;
                }
            }

            if self.in_thinking {
                self.thinking_buffer.push(c);
                // Show progress dots
                if self.thinking_buffer.len() % 3 == 0 {
                    output.push('.');
                }
            } else {
                output.push(c);
            }
        }

        if !output.is_empty() {
            Some(output)
        } else {
            None
        }
    }
}

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = LLMConfig {
        model_name: "o1-preview".to_string(), // Model that supports thinking
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY").unwrap(),
        temperature: 0.7,
        max_tokens: 2048,
    };

    let client = LLMClient::new(config);
    let messages = vec![
        ChatMessage::user("Solve: What is the 15th Fibonacci number?"),
    ];

    let mut tracker = ThinkingTracker::new();
    
    print!("Assistant: ");
    io::stdout().flush()?;

    let response = client.chat_stream(messages, None, |chunk| {
        if let Some(output) = tracker.process_chunk(chunk) {
            print!("{}", output);
            io::stdout().flush().unwrap();
        }
    }).await?;

    println!("\n");
    
    Ok(())
}
```

## Non-Streaming Mode

If you don't want streaming, use the regular `chat` method:

```rust
let client = LLMClient::new(config);
let messages = vec![ChatMessage::user("Hello!")];

// Non-streaming - waits for complete response
let response = client.chat(messages, None).await?;
println!("Assistant: {}", response.content);
```

## API Reference

### `chat_stream` Method

```rust
pub async fn chat_stream<F>(
    &self,
    messages: Vec<ChatMessage>,
    tools: Option<Vec<ToolDefinition>>,
    on_chunk: F,
) -> Result<ChatMessage>
where
    F: FnMut(&str) + Send,
```

**Parameters:**
- `messages`: The conversation messages
- `tools`: Optional tool definitions (for function calling)
- `on_chunk`: Callback function called for each chunk of text

**Returns:**
- `Result<ChatMessage>`: The complete response message after streaming finishes

**Example:**
```rust
let response = client.chat_stream(messages, None, |chunk| {
    print!("{}", chunk);
    io::stdout().flush().unwrap();
}).await?;
```

## Best Practices

### 1. Always Flush Output

```rust
client.chat_stream(messages, None, |chunk| {
    print!("{}", chunk);
    io::stdout().flush().unwrap(); // Important!
}).await?;
```

### 2. Handle Errors Gracefully

```rust
match client.chat_stream(messages, None, |chunk| {
    print!("{}", chunk);
    io::stdout().flush().unwrap();
}).await {
    Ok(response) => {
        println!("\nComplete!");
    }
    Err(e) => {
        eprintln!("\nError: {}", e);
    }
}
```

### 3. Use Appropriate Models

Not all models support thinking tags:
- **OpenAI o1 series**: ✓ Supports thinking tags
- **GPT-4, GPT-3.5**: ✗ No thinking tags
- **Local models**: Depends on model

### 4. Consider Performance

Streaming adds minimal overhead but provides better UX:
- Use streaming for user-facing applications
- Use non-streaming for batch processing
- Use non-streaming when you need the complete response first

## Troubleshooting

### Streaming Not Working

**Problem**: No output appears until response is complete

**Solution**: Make sure you're calling `io::stdout().flush()` in your callback:
```rust
|chunk| {
    print!("{}", chunk);
    io::stdout().flush().unwrap(); // Required!
}
```

### Thinking Tags Not Appearing

**Problem**: Model supports thinking but tags aren't shown

**Possible causes:**
1. Model doesn't actually use thinking tags (check model documentation)
2. Thinking tracker not implemented correctly
3. Model was instructed not to use thinking tags

### Connection Issues

**Problem**: Streaming fails with connection errors

**Solutions:**
- Check internet connection
- Verify API key is correct
- Ensure base_url is correct
- Check if API supports streaming

## Examples

See the `examples/` directory:
- `streaming_chat.rs`: Comprehensive streaming examples
- `direct_llm_usage.rs`: Basic usage patterns

Run examples:
```bash
cargo run --example streaming_chat
```

## Future Enhancements

Planned features:
- Token-by-token streaming control
- Stream cancellation
- Custom chunk processing
- Progress callbacks
- Streaming with tool calls

## Additional Resources

- [Main Documentation](../README.md)
- [API Reference](API.md)
- [Examples](../examples/)
- [OpenAI Streaming Docs](https://platform.openai.com/docs/api-reference/streaming)
