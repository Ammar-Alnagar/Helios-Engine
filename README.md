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
- **🆕 Tool Builder**: Simplified tool creation with builder pattern - wrap any function as a tool without manual trait implementation
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

### 🎯 Start Here
- **[📖 Getting Started](docs/GETTING_STARTED.md)** - Comprehensive guide: installation, configuration, first agent, tools, and CLI

### 🔧 Core Features
- **[🛠️ Tools Guide](docs/TOOLS.md)** - Built-in tools, custom tool creation, and Tool Builder (new `.tools()` syntax!)
- **[🌲 Forest of Agents](docs/FOREST.md)** - Multi-agent systems, coordination, and communication (new `.agents()` syntax!)
- **[🔍 RAG System](docs/RAG.md)** - Retrieval-Augmented Generation with vector stores

### 📖 Reference
- **[📋 API Reference](docs/API.md)** - Complete API documentation
- **[⚙️ Configuration](docs/CONFIGURATION.md)** - Configuration options and local inference setup
- **[✨ Features](docs/FEATURES.md)** - Complete feature overview
- **[🏗️ Architecture](docs/ARCHITECTURE.md)** - System architecture and design
- **[📦 Using as Crate](docs/USING_AS_CRATE.md)** - Library usage guide

📚 **[Full Documentation Index](docs/README.md)** - Complete navigation and updated structure

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
helios-engine = "0.4.1"
tokio = { version = "1.35", features = ["full"] }
```

See **[📖 Getting Started Guide](docs/GETTING_STARTED.md)** for detailed examples and comprehensive tutorial!

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
