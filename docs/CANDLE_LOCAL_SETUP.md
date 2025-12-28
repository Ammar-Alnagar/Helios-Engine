# Running Qwen2.5-0.5B-Instruct Locally with Candle

This guide shows you how to set up and run the Qwen2.5-0.5B-Instruct model locally using the Candle backend with automatic cache loading.

## Prerequisites

1. **Rust installed** (version 1.70 or higher)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Model cached locally** - The model is automatically loaded from your HuggingFace cache

## Step 1: Download Model to Local Cache

First, download the model to your local HuggingFace cache. You only need to do this once:

```bash
huggingface-cli download Qwen/Qwen2.5-0.5B-Instruct --local-dir ~/.cache/huggingface/hub/models--Qwen--Qwen2.5-0.5B-Instruct --local-dir-use-symlinks False
```

Or using Python:
```python
from huggingface_hub import snapshot_download
snapshot_download("Qwen/Qwen2.5-0.5B-Instruct", cache_dir="~/.cache/huggingface")
```

**Verify the download:**
```bash
ls -la ~/.cache/huggingface/hub/models--Qwen--Qwen2.5-0.5B-Instruct/snapshots/*/
```

You should see files like:
- `model.safetensors` (main model file)
- `tokenizer.json` (tokenizer)
- `config.json` (model config)

## Step 2: Create Configuration File

Create a `config.toml` file in your project root (or copy from `config.example.toml`):

```toml
[llm]
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "your-api-key-here"
temperature = 0.7
max_tokens = 2048

[candle]
# Qwen2.5-0.5B-Instruct configuration
huggingface_repo = "Qwen/Qwen2.5-0.5B-Instruct"
model_file = "model.safetensors"
context_size = 32768
temperature = 0.7
max_tokens = 2048
use_gpu = true
```

### Configuration Options

- **`huggingface_repo`**: The HuggingFace model repository (e.g., `Qwen/Qwen2.5-0.5B-Instruct`)
- **`model_file`**: The model file name in the repository (typically `model.safetensors`)
- **`context_size`**: Maximum context length (Qwen2.5-0.5B-Instruct supports up to 32768)
- **`temperature`**: Controls randomness (0.0-1.0, lower = more deterministic)
- **`max_tokens`**: Maximum tokens to generate per request
- **`use_gpu`**: Whether to use GPU if available (set to `false` for CPU-only)

## Step 3: Build the Project

Build with the `candle` feature enabled:

```bash
cargo build --features candle --release
```

This will compile the project with Candle ML framework support.

## Step 4: Run the Application

### Option A: Using the binary directly

```bash
./target/release/helios-engine
```

### Option B: Using cargo run

```bash
cargo run --features candle --release
```

### Option C: Running specific examples

```bash
# Basic chat example
cargo run --example basic_chat --features candle --release

# Direct LLM usage
cargo run --example direct_llm_usage --features candle --release

# Agent with tools
cargo run --example agent_with_tools --features candle --release
```

## Step 5: Use the Model in Your Code

### Simple Chat Example

```rust
use helios_engine::{Config, Agent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config from file
    let config = Config::from_file("config.toml")?;
    
    // Create an agent with Candle backend
    let mut agent = Agent::new(config).await?;
    
    // Send a message
    let response = agent.chat("Hello, what is Rust?").await?;
    println!("Response: {}", response);
    
    Ok(())
}
```

## Model Information

**Qwen2.5-0.5B-Instruct**
- **Parameters**: 0.5 billion
- **Context Window**: 32,768 tokens
- **Format**: Chat-optimized (Instruct)
- **Size**: ~400-600 MB (depending on precision)
- **Speed**: Very fast on CPU and GPU
- **Language**: English + Chinese

This is a lightweight model ideal for:
- Local development and testing
- Edge devices and resource-constrained environments
- Fast inference with reasonable quality

## Custom Cache Location

If you want to use a different cache location:

```bash
# Set custom HuggingFace cache directory
export HF_HOME=/path/to/custom/cache

# Then run your application
cargo run --features candle --release
```

## Troubleshooting

### "Model not found in cache"

Make sure you've downloaded the model:
```bash
huggingface-cli download Qwen/Qwen2.5-0.5B-Instruct
```

Verify the cache structure:
```bash
ls ~/.cache/huggingface/hub/models--Qwen--Qwen2.5-0.5B-Instruct/snapshots/*/model.safetensors
```

### "Failed to initialize tokenizer"

The tokenizer file must be in the same snapshot directory as the model. Verify:
```bash
ls ~/.cache/huggingface/hub/models--Qwen--Qwen2.5-0.5B-Instruct/snapshots/*/tokenizer.json
```

### Out of Memory

If you get OOM errors:
- Reduce `context_size` in config.toml
- Reduce `max_tokens`
- Set `use_gpu = false` to use CPU (slower but uses less GPU memory)

### Slow Performance

If inference is slow:
- Ensure `use_gpu = true` if you have a CUDA-capable GPU
- Install CUDA libraries if using GPU
- Consider reducing `context_size` and `max_tokens`

## Alternative Models

You can also use other models with Candle. Just change the `huggingface_repo` and ensure the model file is in your cache:

```toml
[candle]
# Qwen2-7B (larger, better quality)
huggingface_repo = "Qwen/Qwen2-7B-Instruct"
model_file = "model.safetensors"

# Or Llama
huggingface_repo = "meta-llama/Llama-2-7b-chat"
model_file = "model.safetensors"

# Or Gemma
huggingface_repo = "google/gemma-7b-it"
model_file = "model.safetensors"
```

## Performance Tips

1. **First run is slow**: Model loading and compilation takes time on first use
2. **Use GPU**: Enable `use_gpu = true` for 10-20x speedup
3. **Batch requests**: Process multiple requests together
4. **Cache model**: Keep the model in cache to avoid re-downloading

## Next Steps

- Check out `examples/` directory for more usage patterns
- Read the [Candle documentation](https://github.com/huggingface/candle)
- Explore other examples like `agent_with_tools.rs`, `forest_of_agents.rs`, etc.

