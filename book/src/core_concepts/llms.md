# LLMs

The `LLMClient` is the primary interface for interacting with Large Language Models (LLMs) in the Helios Engine. It provides a unified API for both remote LLMs (like OpenAI) and local LLMs (via `llama.cpp`).

## The `LLMClient`

The `LLMClient` is responsible for sending requests to the LLM and receiving responses. It can be created with either a `Remote` or `Local` provider type.

### Creating an `LLMClient`

Here's how to create an `LLMClient` with a remote provider:

```rust
use helios_engine::{llm::{LLMClient, LLMProviderType}, config::LLMConfig};

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

    Ok(())
}
```

And here's how to create an `LLMClient` with a local provider:

```rust
# #[cfg(feature = "local")]
use helios_engine::{llm::{LLMClient, LLMProviderType}, config::LocalConfig};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let local_config = LocalConfig {
        huggingface_repo: "unsloth/Qwen3-0.6B-GGUF".to_string(),
        model_file: "Qwen3-0.6B-Q4_K_M.gguf".to_string(),
        temperature: 0.7,
        max_tokens: 2048,
    };

    let client = LLMClient::new(LLMProviderType::Local(local_config)).await?;

    Ok(())
}
```

**Note:** To use the local provider, you must install Helios Engine with the `local` feature enabled.

## Sending Requests

Once you have an `LLMClient`, you can send requests to the LLM using the `chat` method.

### Simple Chat

Here's a simple example of how to send a chat request:

```rust
use helios_engine::{llm::{LLMClient, LLMProviderType}, config::LLMConfig, ChatMessage};

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

    let messages = vec![ChatMessage::user("Hello, world!")];
    let response = client.chat(messages, None, None, None, None).await?;

    println!("Assistant: {}", response.content);

    Ok(())
}
```

### Streaming Responses

The `LLMClient` also supports streaming responses. Here's an example of how to use the `chat_stream` method:

```rust
use helios_engine::{llm::{LLMClient, LLMProviderType}, config::LLMConfig, ChatMessage};

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

    let messages = vec![ChatMessage::user("Hello, world!")];
    let response = client.chat_stream(messages, None, None, None, None, |chunk| {
        print!("{}", chunk);
    }).await?;

    Ok(())
}
```
