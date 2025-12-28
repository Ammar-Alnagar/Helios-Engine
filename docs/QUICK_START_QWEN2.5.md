# Quick Start: Qwen2.5-0.5B-Instruct with Candle

## TL;DR - Get running in 3 steps

### 1. Download Model (one-time setup)
```bash
huggingface-cli download Qwen/Qwen2.5-0.5B-Instruct
```

### 2. Create config.toml
```toml
[llm]
model_name = "gpt-3.5-turbo"
base_url = "https://api.openai.com/v1"
api_key = "your-api-key-here"
temperature = 0.7
max_tokens = 2048

[candle]
huggingface_repo = "Qwen/Qwen2.5-0.5B-Instruct"
model_file = "model.safetensors"
context_size = 32768
temperature = 0.7
max_tokens = 2048
use_gpu = true
```

### 3. Run
```bash
cargo run --features candle --release
```

---

## Key Commands

### Build
```bash
# Development build
cargo build --features candle

# Optimized release build (recommended)
cargo build --features candle --release
```

### Run Examples
```bash
# Basic chat
cargo run --example basic_chat --features candle --release

# Direct LLM usage
cargo run --example direct_llm_usage --features candle --release

# Agent with tools
cargo run --example agent_with_tools --features candle --release

# Complete demo
cargo run --example complete_demo --features candle --release
```

### Verify Cache
```bash
# Check if model is cached
ls ~/.cache/huggingface/hub/models--Qwen--Qwen2.5-0.5B-Instruct/snapshots/*/model.safetensors

# Download if not cached
huggingface-cli download Qwen/Qwen2.5-0.5B-Instruct

# Use custom cache location
export HF_HOME=/path/to/custom/cache
```

---

## Cache Structure

The model is automatically loaded from:
```
~/.cache/huggingface/hub/models--Qwen--Qwen2.5-0.5B-Instruct/
└── snapshots/
    └── {hash}/
        ├── model.safetensors      ← Main model
        ├── tokenizer.json         ← Tokenizer
        ├── config.json            ← Config
        ├── generation_config.json
        └── ...
```

---

## How Cache Loading Works

1. ✅ Application starts → looks in `~/.cache/huggingface/`
2. ✅ Finds `models--Qwen--Qwen2.5-0.5B-Instruct` → loads model & tokenizer
3. ✅ If not found → downloads from HuggingFace Hub

**No configuration needed** - it's automatic!

---

## Configuration Reference

| Option | Value | Description |
|--------|-------|-------------|
| `huggingface_repo` | `Qwen/Qwen2.5-0.5B-Instruct` | Model repository |
| `model_file` | `model.safetensors` | Model file name |
| `context_size` | `32768` | Max context (tokens) |
| `temperature` | `0.7` | Randomness (0.0-1.0) |
| `max_tokens` | `2048` | Max output (tokens) |
| `use_gpu` | `true` | Use GPU if available |

---

## Alternative Models

```toml
# Qwen2 (larger, better quality)
huggingface_repo = "Qwen/Qwen2-7B-Instruct"

# Llama 2
huggingface_repo = "meta-llama/Llama-2-7b-chat"

# Gemma
huggingface_repo = "google/gemma-7b-it"

# Mistral
huggingface_repo = "mistralai/Mistral-7B-Instruct-v0.1"
```

All use the same cache loading mechanism!

---

## Performance

| Setting | Speed | Memory | Quality |
|---------|-------|--------|---------|
| CPU | Slow | Low | - |
| GPU (`use_gpu=true`) | Fast | Medium | - |
| Large context | - | High | - |
| Small context | - | Low | - |

**Tip**: First run is slow (compilation + model load). Subsequent runs are fast!

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Model not found | Run: `huggingface-cli download Qwen/Qwen2.5-0.5B-Instruct` |
| Out of memory | Reduce `context_size` or `max_tokens` |
| Slow inference | Enable `use_gpu = true` |
| Can't find tokenizer | Delete cache and re-download |

---

## Example Usage in Code

```rust
use helios_engine::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file("config.toml")?;
    let mut agent = helios_engine::Agent::new(config).await?;
    
    let response = agent.chat("What is Rust?").await?;
    println!("{}", response);
    
    Ok(())
}
```

---

## Next Steps

- ✅ Read `CANDLE_LOCAL_SETUP.md` for detailed guide
- ✅ Check `examples/` for more usage patterns
- ✅ Try different models from config alternatives
- ✅ Explore agent capabilities with tools

