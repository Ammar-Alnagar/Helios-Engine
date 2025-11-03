# 🔥 Helios Engine - LLM Agent Framework


<p align="center">
  <img src="Helios_Engine_Logo.png" alt="Helios Engine Logo" width="350"/>
</p>

[![Crates.io](https://img.shields.io/crates/v/helios-engine.svg)](https://crates.io/crates/helios-engine)
[![docs.rs](https://docs.rs/helios-engine/badge.svg)](https://docs.rs/helios-engine)
[![downloads](https://img.shields.io/crates/d/helios-engine.svg)](https://crates.io/crates/helios-engine)
[![issues](https://img.shields.io/github/issues/Ammar-Alnagar/Helios-Engine.svg)](https://github.com/Ammar-Alnagar/Helios-Engine/issues)
[![stars](https://img.shields.io/github/stars/Ammar-Alnagar/Helios-Engine.svg)](https://github.com/Ammar-Alnagar/Helios-Engine/stargazers)
[![last commit](https://img.shields.io/github/last-commit/Ammar-Alnagar/Helios-Engine.svg)](https://github.com/Ammar-Alnagar/Helios-Engine/commits/main)

**Helios Engine** is a powerful and flexible Rust framework for building LLM-powered agents with tool support, streaming chat capabilities, and easy configuration management. Create intelligent agents that can interact with users, call tools, and maintain conversation context - with both online and offline local model support.

## 🚀 Key Features

- **🆕 Forest of Agents**: Multi-agent collaboration system where agents can communicate, delegate tasks, and share context
- **Agent System**: Create multiple agents with different personalities and capabilities
- **Tool Registry**: Extensible tool system for adding custom functionality
- **Extensive Tool Suite**: 16+ built-in tools including web scraping, JSON parsing, timestamp operations, file I/O, shell commands, HTTP requests, system info, and text processing
- **🆕 RAG System**: Retrieval-Augmented Generation with vector stores (InMemory and Qdrant)
- **Streaming Support**: True real-time response streaming for both remote and local models with immediate token delivery
- **Local Model Support**: Run local models offline using llama.cpp with HuggingFace integration (optional `local` feature)
- **HTTP Server & API**: Expose OpenAI-compatible API endpoints with full parameter support
- **Dual Mode Support**: Auto, online (remote API), and offline (local) modes
- **CLI & Library**: Use as both a command-line tool and a Rust library crate
- **🆕 Feature Flags**: Optional `local` feature for offline model support - build only what you need!

## 📚 Documentation

- **[📖 Getting Started](docs/QUICKSTART.md)** - 5-minute setup guide to get Helios running
- **[🛠️ Installation](docs/INSTALLATION.md)** - Complete installation instructions and feature flags
- **[💻 CLI Usage](docs/USAGE.md)** - Command-line interface and common usage patterns
- **[⚙️ Configuration](docs/CONFIGURATION.md)** - Configuration options and local inference setup
- **[🔧 Tools](docs/TOOLS.md)** - Built-in tools and creating custom tools
- **[🆕 Advanced Features](docs/ADVANCED.md)** - RAG, Forest of Agents, and advanced capabilities
- **[📋 API Reference](docs/API.md)** - Complete API documentation
- **[🏗️ Architecture](docs/ARCHITECTURE.md)** - System architecture and design principles

📚 **[Full Documentation Index](docs/README.md)** - Complete guide to all available documentation

## 🏃‍♂️ Quick Start

### Install CLI Tool
```bash
# Install without local model support (lighter, faster install)
cargo install helios-engine

# Install with local model support (enables offline mode with llama-cpp-2)
cargo install helios-engine --features local
```

### Basic Usage
```bash
# Initialize configuration
helios-engine init

# Start interactive chat
helios-engine chat

# Ask a quick question
helios-engine ask "What is Rust?"
```

### As a Library Crate
Add to your `Cargo.toml`:
```toml
[dependencies]
helios-engine = "0.3.7"
tokio = { version = "1.35", features = ["full"] }
```

See **[📖 Quick Start Guide](docs/QUICKSTART.md)** for detailed examples!

## 📁 Project Structure

```
helios-engine/
├── src/                    # Source code
├── examples/               # Example applications
├── docs/                   # Documentation
├── tests/                  # Integration tests
├── Cargo.toml             # Project configuration
└── README.md              # This file
```

## 🤝 Contributing

We welcome contributions! See **[Contributing Guide](docs/README.md#contributing-to-documentation)** for details on:
- Development setup
- Code standards
- Documentation guidelines
- Testing procedures

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

Made with ❤️ in Rust
