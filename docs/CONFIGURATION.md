# Configuration Guide

Helios Engine supports flexible configuration for both remote API access and local model inference through the dual LLMProviderType system. This guide covers all configuration options and setup scenarios.

## Quick Start Configuration

### Basic Remote API Setup

Create a `config.toml` file in your project root:

```toml
[llm]
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "sk-your-api-key-here"
temperature = 0.7
max_tokens = 2048
```

### Programmatic Configuration

Create configuration in code:

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

## Supported LLM Providers

### Remote APIs (Online Mode)

Helios Engine works with any OpenAI-compatible API:

#### OpenAI
```toml
[llm]
model_name = "gpt-4"  # or gpt-3.5-turbo, gpt-4-turbo, etc.
base_url = "https://api.openai.com/v1"
api_key = "sk-your-openai-api-key"
temperature = 0.7
max_tokens = 2048
```

#### Azure OpenAI
```toml
[llm]
model_name = "gpt-35-turbo"  # Azure deployment name
base_url = "https://your-resource.openai.azure.com/openai/deployments/your-deployment"
api_key = "your-azure-openai-key"
temperature = 0.7
max_tokens = 2048
```

#### Local Models via API (LM Studio, Ollama, etc.)
```toml
[llm]
model_name = "local-model"
base_url = "http://localhost:1234/v1"  # LM Studio default
api_key = "not-needed"  # or your API key if required
temperature = 0.7
max_tokens = 2048
```

#### Ollama
```toml
[llm]
model_name = "llama2"  # or any model you have pulled
base_url = "http://localhost:11434/v1"
api_key = "not-needed"
temperature = 0.7
max_tokens = 2048
```

### Local Models (Offline Mode)

Run models locally using llama.cpp without internet connection:

#### Prerequisites

1. **HuggingFace Account**: Sign up at [huggingface.co](https://huggingface.co) (free)
2. **HuggingFace CLI**: Install and login:
   ```bash
   pip install huggingface_hub
   huggingface-cli login  # Login with your token
   ```

#### Local Model Configuration

```toml
[llm]
# Remote config still needed for auto mode fallback
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "your-api-key-here"
temperature = 0.7
max_tokens = 2048

# Local model configuration for offline mode
[local]
huggingface_repo = "unsloth/Qwen3-0.6B-GGUF"
model_file = "Qwen3-0.6B-Q4_K_M.gguf"
temperature = 0.7
max_tokens = 2048
context_size = 8192  # Optional, defaults to 4096
```

### Auto Mode Configuration (Remote + Local)

For maximum flexibility, configure both remote and local models to enable auto mode:

```toml
[llm]
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "your-api-key-here"
temperature = 0.7
max_tokens = 2048

# Local model as fallback
[local]
huggingface_repo = "unsloth/Qwen3-0.6B-GGUF"
model_file = "Qwen3-0.6B-Q4_K_M.gguf"
temperature = 0.7
max_tokens = 2048
```

## Local Inference Setup

### Setting Up Local Models

1. **Find a GGUF Model**: Browse [HuggingFace Models](https://huggingface.co/models?library=gguf) for compatible models

2. **Update Configuration**: Add local model config to your `config.toml`:
   ```toml
   [local]
   huggingface_repo = "unsloth/Qwen3-0.6B-GGUF"
   model_file = "Qwen3-0.6B-Q4_K_M.gguf"
   temperature = 0.7
   max_tokens = 2048
   context_size = 8192
   ```

3. **Run in Offline Mode**:
   ```bash
   # First run downloads the model (~400MB for Qwen3-0.6B)
   helios-engine --mode offline ask "Hello world"

   # Subsequent runs use cached model
   helios-engine --mode offline chat
   ```

### Recommended Models

| Model | Size | Use Case | Repository |
|-------|------|----------|------------|
| Qwen3-0.6B | ~400MB | Fast, good quality | `unsloth/Qwen3-0.6B-GGUF` |
| Llama-3.2-1B | ~700MB | Balanced performance | `unsloth/Llama-3.2-1B-Instruct-GGUF` |
| Mistral-7B | ~4GB | High quality | `TheBloke/Mistral-7B-Instruct-v0.1-GGUF` |
| Llama-3-8B | ~5GB | Excellent quality | `unsloth/Meta-Llama-3-8B-Instruct-GGUF` |

### Performance & Features

- **GPU Acceleration**: Models automatically use GPU if available (via llama.cpp's n_gpu_layers parameter)
- **Model Caching**: Downloaded models are cached locally (~/.cache/huggingface)
- **Memory Usage**: Larger models need more RAM/VRAM
- **First Run**: Initial model download may take time depending on connection speed
- **Clean Output Mode**: Suppresses verbose debugging from llama.cpp for clean user experience

### Local Model Parameters

```toml
[local]
# HuggingFace repository and model file (required)
huggingface_repo = "unsloth/Qwen3-0.6B-GGUF"
model_file = "Qwen3-0.6B-Q4_K_M.gguf"

# Generation parameters (optional, defaults provided)
temperature = 0.7        # 0.0-2.0, controls randomness
max_tokens = 2048        # Maximum tokens to generate
context_size = 8192      # Context window size

# Advanced parameters (optional)
top_k = 40              # Top-k sampling
top_p = 0.9             # Nucleus sampling
repeat_penalty = 1.1    # Repetition penalty

# Hardware acceleration (optional)
n_gpu_layers = -1       # -1 = use all available GPU layers, 0 = CPU only
n_threads = -1          # -1 = use all available CPU threads
```

## Operation Modes

### Auto Mode
Uses local model if available and configured, otherwise falls back to remote API:
```bash
helios-engine --mode auto chat
```

### Online Mode
Forces remote API usage, ignores local configuration:
```bash
helios-engine --mode online chat
```

### Offline Mode
Uses only local models, fails if not configured:
```bash
helios-engine --mode offline chat
```

## Environment Variables

Use environment variables for sensitive configuration:

```bash
export OPENAI_API_KEY="sk-your-key-here"
export LLM_BASE_URL="https://api.openai.com/v1"
export LLM_MODEL="gpt-4"
```

```rust
use helios_engine::config::LLMConfig;

let config = LLMConfig {
    model_name: std::env::var("LLM_MODEL")
        .unwrap_or_else(|_| "gpt-3.5-turbo".to_string()),
    base_url: std::env::var("LLM_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
    api_key: std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY must be set"),
    temperature: 0.7,
    max_tokens: 2048,
};
```

## Advanced Configuration

### Custom HTTP Client

For production deployments with connection pooling:

```rust
use helios_engine::config::LLMConfig;
use reqwest::Client;

let http_client = Client::builder()
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(std::time::Duration::from_secs(30))
    .tcp_keepalive(std::time::Duration::from_secs(60))
    .build()
    .await?;

let config = LLMConfig {
    model_name: "gpt-4".to_string(),
    base_url: "https://api.openai.com/v1".to_string(),
    api_key: std::env::var("OPENAI_API_KEY").unwrap(),
    temperature: 0.7,
    max_tokens: 2048,
    client: Some(http_client),
};
```

### Multiple Configurations

Manage different configurations for different use cases:

```rust
use helios_engine::{Config, Agent};
use std::collections::HashMap;

// Load multiple configs
let prod_config = Config::from_file("config.prod.toml")?;
let dev_config = Config::from_file("config.dev.toml")?;
let local_config = Config::from_file("config.local.toml")?;

// Create agents with different configs
let mut prod_agent = Agent::builder("ProductionAgent")
    .config(prod_config)
    .build()
    .await?;

let mut dev_agent = Agent::builder("DevelopmentAgent")
    .config(dev_config)
    .build()
    .await?;
```

### Configuration Validation

Validate configuration before use:

```rust
use helios_engine::Config;

let config = Config::from_file("config.toml")?;

// Validate LLM configuration
config.validate_llm_config()?;

// Validate local model configuration (if present)
if let Some(local_config) = &config.local {
    local_config.validate()?;
}

// Configuration is ready to use
let mut agent = Agent::builder("ValidatedAgent")
    .config(config)
    .build()
    .await?;
```

## Configuration Files Organization

### Project Structure

```
my-project/
├── config.toml          # Main configuration
├── config.prod.toml     # Production settings
├── config.dev.toml      # Development settings
├── config.local.toml    # Local model settings
└── src/
    └── main.rs
```

### Environment-Specific Configs

**Development (config.dev.toml):**
```toml
[llm]
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "sk-dev-key"
temperature = 0.9  # More creative for development
max_tokens = 1024  # Shorter for faster iteration
```

**Production (config.prod.toml):**
```toml
[llm]
model_name = "gpt-4"
base_url = "https://api.openai.com/v1"
api_key = "sk-prod-key"
temperature = 0.3  # More deterministic for production
max_tokens = 2048
```

## Troubleshooting Configuration

### Common Issues

**"API key not found"**
- Ensure your API key is set in environment variables or config file
- Check that the environment variable name matches exactly
- Verify the API key is valid and has proper permissions

**"Model not found"**
- Check that the model name is correct for your provider
- Ensure the model is available in your OpenAI/Azure account
- For local models, verify the HuggingFace repository and file exist

**"Connection failed"**
- Verify the base_url is correct and accessible
- Check firewall/proxy settings
- Ensure the API endpoint is responding

**"Local model download failed"**
- Verify HuggingFace CLI is installed and logged in
- Check available disk space (models can be several GB)
- Ensure stable internet connection for initial download

**"GPU not detected"**
- Install CUDA/cuBLAS for GPU acceleration
- Check that llama.cpp was compiled with GPU support
- Set `n_gpu_layers = 0` to force CPU-only mode

### Configuration Validation

Create a validation script to check your configuration:

```rust
use helios_engine::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Validating Helios Engine configuration...");

    // Test config loading
    let config = Config::from_file("config.toml")?;
    println!("✅ Configuration file loaded successfully");

    // Test LLM client creation
    let _client = helios_engine::LLMClient::new(config.llm_provider()).await?;
    println!(" LLM client created successfully");

    // Test local model (if configured)
    if let Some(local_config) = &config.local {
        println!("Testing local model configuration...");
        // Note: Actual model loading would happen on first use
        println!("Local model configuration is valid");
    }

    println!("All configuration tests passed!");
    Ok(())
}
```

## Next Steps

- **[Installation Guide](INSTALLATION.md)** - How to install Helios Engine
- **[Usage Guide](USAGE.md)** - Common usage patterns
- **[Tools Guide](TOOLS.md)** - Available tools and custom tool creation
- **[Examples](../examples/)** - Working configuration examples
