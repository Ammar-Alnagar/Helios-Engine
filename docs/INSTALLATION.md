# Installation Guide

Helios Engine can be used both as a **command-line tool** and as a **library crate** in your Rust projects.

## As a CLI Tool (Recommended for Quick Start)

### Install from Crates.io

Install globally using Cargo:

```bash
# Install without local model support (lighter, faster install)
cargo install helios-engine

# Install with local model support (enables offline mode with llama-cpp-2)
cargo install helios-engine --features local
```

Then use anywhere:

```bash
# Initialize configuration
helios-engine init

# Start interactive chat (default command)
helios-engine
# or explicitly
helios-engine chat

# Ask a quick question
helios-engine ask "What is the capital of France?"

# Get help
helios-engine --help
```

### Additional CLI Usage Examples

```bash
# Use offline mode with local models (no internet required)
helios-engine --mode offline chat

# Use online mode (forces remote API usage)
helios-engine --mode online chat

# Auto mode (uses local if configured, otherwise remote)
helios-engine --mode auto chat

# Verbose logging for debugging
helios-engine --verbose chat

# Custom system prompt
helios-engine chat --system-prompt "You are a Python expert"

# One-off question with custom config
helios-engine --config /path/to/config.toml ask "Calculate 15 * 7"

# Serve OpenAI-compatible API endpoints
helios-engine serve --port 8000 --host 127.0.0.1

# Serve on all interfaces
helios-engine serve --host 0.0.0.0
```

## As a Library Crate

Add Helios-Engine to your `Cargo.toml`:

```toml
[dependencies]
# Without local model support (lighter dependency)
helios-engine = "0.3.7"
tokio = { version = "1.35", features = ["full"] }

# OR with local model support for offline inference
helios-engine = { version = "0.3.7", features = ["local"] }
tokio = { version = "1.35", features = ["full"] }
```

Or use a local path during development:

```toml
[dependencies]
helios-engine = { path = "../helios-engine" }
tokio = { version = "1.35", features = ["full"] }
```

## Build from Source

Clone and build the project:

```bash
git clone https://github.com/Ammar-Alnagar/Helios-Engine.git
cd Helios-Engine

# Build without local model support
cargo build --release

# OR build with local model support
cargo build --release --features local

# Install locally (without local support)
cargo install --path .

# OR install with local model support
cargo install --path . --features local
```

## 🚩 Feature Flags

Helios Engine supports optional feature flags to control which dependencies are included in your build. This allows you to create lighter builds when you don't need certain functionality.

### Available Features

#### `local` - Local Model Support

Enables offline inference using local models via llama-cpp-2. When disabled, the engine only supports remote API calls, resulting in:

- **Faster compilation times** - No need to build llama-cpp-2 and its dependencies
- **Smaller binary size** - Excludes large native libraries
- **Simpler dependencies** - Reduces the dependency tree significantly

**Enables:**
- `LocalLLMProvider` - Run models locally using llama.cpp
- `LocalConfig` - Configuration for local model setup
- Offline mode (`--mode offline`) in the CLI
- HuggingFace model downloading and caching

**When to use:**
- ✅ Use `--features local` if you need offline inference or want to run models locally
- ❌ Skip it if you only use remote APIs (OpenAI, Azure, etc.) for faster builds

**Example:**

```bash
# Without local support (lightweight, remote API only)
cargo install helios-engine
cargo build --release

# With local support (includes llama-cpp-2 for offline inference)
cargo install helios-engine --features local
cargo build --release --features local
```

**In Cargo.toml:**

```toml
# Remote API only
[dependencies]
helios-engine = "0.3.7"

# With local model support
[dependencies]
helios-engine = { version = "0.3.7", features = ["local"] }
```

## System Requirements

### For Remote API Usage Only
- Rust 1.70+
- Internet connection for API calls

### For Local Model Support
- Rust 1.70+
- C++ compiler (for llama-cpp-2)
- At least 4GB RAM (8GB+ recommended for larger models)
- Sufficient disk space for model storage (~4GB+ for typical models)

### Operating System Support
- Linux (primary development platform)
- macOS
- Windows (with appropriate C++ toolchain)

## Troubleshooting

### Build Issues
If you encounter build issues with the `local` feature:

1. Ensure you have a C++ compiler installed:
   - Ubuntu/Debian: `sudo apt install build-essential`
   - macOS: Install Xcode command line tools
   - Windows: Install Visual Studio Build Tools

2. Try building without the local feature first:
   ```bash
   cargo build --release
   ```

3. For local model support, ensure you have sufficient RAM and disk space

### Runtime Issues
- **API Key Not Found**: Ensure your `config.toml` has the correct API key
- **Model Download Issues**: Check your internet connection and HuggingFace access
- **Port Already in Use**: Choose a different port for the serve command

## Next Steps

After installation, check out:
- **[Quick Start Guide](QUICKSTART.md)** - Get up and running in 5 minutes
- **[Configuration Guide](USAGE.md#configuration)** - Set up your API keys and models
- **[Examples](../examples/)** - See Helios in action
