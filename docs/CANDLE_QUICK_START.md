# Candle Backend Quick Start Guide

## Enabling the Candle Feature

To use the Candle backend for local model inference, add the feature flag when building:

```bash
cargo build --features candle
cargo check --features candle
cargo test --features candle
```

Or in your `Cargo.toml`:

```toml
[dependencies]
helios-engine = { version = "0.5.5", features = ["candle"] }
```

## Basic Usage

### 1. Configuration

Create a `CandleConfig` with your model details:

```rust
use helios_engine::CandleConfig;

let config = CandleConfig {
    huggingface_repo: "unsloth/Qwen2-7B".to_string(),
    model_file: "model.safetensors".to_string(),
    context_size: 4096,
    temperature: 0.7,
    max_tokens: 2048,
    use_gpu: true,
};
```

### 2. Create an LLM Client

```rust
use helios_engine::{LLMClient, ChatMessage};
use helios_engine::llm::LLMProviderType;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = CandleConfig { /* ... */ };
    let client = LLMClient::new(LLMProviderType::Candle(config)).await?;
    
    let messages = vec![
        ChatMessage::system("You are a helpful assistant."),
        ChatMessage::user("Hello! How are you?"),
    ];
    
    let response = client.chat(messages, None, None, None, None).await?;
    println!("Assistant: {}", response.content);
    
    Ok(())
}
```

### 3. Using with Agents

```rust
use helios_engine::{Agent, Config, CandleConfig};
use helios_engine::llm::LLMProviderType;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let mut config = Config::from_file("config.toml")?;
    
    // Override with Candle provider
    config.candle = Some(CandleConfig {
        huggingface_repo: "unsloth/Qwen2-7B".to_string(),
        model_file: "model.safetensors".to_string(),
        context_size: 4096,
        temperature: 0.7,
        max_tokens: 2048,
        use_gpu: true,
    });
    
    let mut agent = Agent::builder("MyAgent")
        .build()
        .await?;
    
    let response = agent.chat("What is Rust?").await?;
    println!("Agent: {}", response);
    
    Ok(())
}
```

## Supported Models

The Candle backend automatically detects and optimizes for the following model families:

### Qwen Family
- Qwen (base)
- Qwen2
- Qwen3
- Qwen MOE variants

**Format**: `<|im_start|>role\ncontent\n<|im_end|>`

```rust
CandleConfig {
    huggingface_repo: "unsloth/Qwen2-7B".to_string(),
    model_file: "model.safetensors".to_string(),
    // ...
}
```

### Llama Family
- Llama 2
- Llama (base)

**Format**: `[INST]...[/INST]`

```rust
CandleConfig {
    huggingface_repo: "meta-llama/Llama-2-7b".to_string(),
    model_file: "model.safetensors".to_string(),
    // ...
}
```

### Gemma Family
- Gemma
- Gemma 2

**Format**: `<start_of_turn>role\ncontent\n<end_of_turn>`

```rust
CandleConfig {
    huggingface_repo: "google/gemma-7b".to_string(),
    model_file: "model.safetensors".to_string(),
    // ...
}
```

### Mistral
**Format**: `[INST]...[/INST]`

```rust
CandleConfig {
    huggingface_repo: "mistralai/Mistral-7B".to_string(),
    model_file: "model.safetensors".to_string(),
    // ...
}
```

## Configuration File Example

Create a `config.toml`:

```toml
[llm]
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "your-api-key"
temperature = 0.7
max_tokens = 2048

[candle]
huggingface_repo = "unsloth/Qwen2-7B"
model_file = "model.safetensors"
context_size = 4096
temperature = 0.7
max_tokens = 2048
use_gpu = true
```

Then load it:

```rust
let config = Config::from_file("config.toml")?;
if let Some(candle_config) = config.candle {
    let client = LLMClient::new(LLMProviderType::Candle(candle_config)).await?;
}
```

## GPU Support

The Candle backend automatically detects and uses GPU acceleration when available:

- **CUDA**: For NVIDIA GPUs
- **Metal**: For Apple Silicon (M1/M2/M3)
- **CPU**: Fallback for all platforms

Control GPU usage with the `use_gpu` flag:

```rust
CandleConfig {
    use_gpu: true,   // Try to use GPU, fallback to CPU
    // ...
}
```

## Model Download

Models are automatically downloaded from HuggingFace Hub on first use. They are cached in:

- Linux/Mac: `~/.cache/huggingface/hub/`
- Windows: `%USERPROFILE%\.cache\huggingface\hub\`

## Performance Tips

1. **Batch Size**: For better throughput with multiple requests, consider batching
2. **Context Size**: Smaller context sizes use less memory but may limit response quality
3. **Quantization**: Use quantized models for faster inference and lower memory usage
4. **GPU**: Enable GPU for significant speedups on large models

## Troubleshooting

### Model Not Found
- Ensure the `huggingface_repo` is correct
- Check internet connectivity for model download
- Verify the model file exists in the repository

### GPU Not Detected
- Check CUDA installation (for NVIDIA)
- Verify GPU drivers are up to date
- Set `use_gpu: false` to force CPU mode

### Memory Issues
- Reduce `context_size`
- Use quantized models
- Enable GPU acceleration if available

## Examples

See the `examples/` directory for complete working examples:

- `examples/direct_llm_usage.rs` - Direct LLM calls
- `examples/agent_with_tools.rs` - Agent with tools
- `examples/streaming_chat.rs` - Streaming responses

Run with:

```bash
cargo run --example direct_llm_usage --features candle
```

## What's Next?

Check the main documentation for:
- Building agents with tools
- Multi-agent systems
- RAG (Retrieval-Augmented Generation)
- Custom tool creation
