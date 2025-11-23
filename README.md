# Helios Engine - LLM Agent Framework


<p align="center">
  <img src="Helios_Engine_Logo.png" alt="Helios Engine Logo" width="350"/>
</p>

[![Crates.io](https://img.shields.io/crates/v/helios-engine.svg)](https://crates.io/crates/helios-engine)
[![docs.rs](https://docs.rs/helios-engine/badge.svg)](https://docs.rs/helios-engine)
[![Book](https://img.shields.io/badge/book-online-brightgreen.svg)](https://helios-engine.vercel.app/)
[![downloads](https://img.shields.io/crates/d/helios-engine.svg)](https://crates.io/crates/helios-engine)
[![issues](https://img.shields.io/github/issues/Ammar-Alnagar/Helios-Engine.svg)](https://github.com/Ammar-Alnagar/Helios-Engine/issues)
[![stars](https://img.shields.io/github/stars/Ammar-Alnagar/Helios-Engine.svg)](https://github.com/Ammar-Alnagar/Helios-Engine/stargazers)
[![last commit](https://img.shields.io/github/last-commit/Ammar-Alnagar/Helios-Engine.svg)](https://github.com/Ammar-Alnagar/Helios-Engine/commits/main)
[![Website](https://img.shields.io/badge/Website-Online-brightgreen.svg)](https://helios-engine.vercel.app/)
![Release](https://img.shields.io/github/release-date/Ammar-Alnagar/Helios-Engine.svg)

**Helios Engine** is a powerful and flexible Rust framework for building LLM-powered agents with tool support, streaming chat capabilities, and easy configuration management. Create intelligent agents that can interact with users, call tools, and maintain conversation context - with both online and offline local model support.

## Key Features

- **🆕 ReAct Mode**: Enable agents to reason and plan before taking actions with a simple `.react()` call - includes custom reasoning prompts for domain-specific tasks
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

## Documentation

###  Online Resources
- **[Official Website](https://helios-engine.vercel.app/)** - Complete interactive documentation with tutorials, guides, and examples
- **[Official Book ](https://ammar-alnagar.github.io/Helios-Engine/)** - Comprehensive guide to Helios Engine
- **[ API Reference](https://docs.rs/helios-engine)** - Detailed API documentation on docs.rs

###  Quick Links
- **[Getting Started](https://helios-engine.vercel.app/getting_started/installation.html)** - Installation and first steps
- **[Core Concepts](https://helios-engine.vercel.app/core_concepts/agents.html)** - Agents, LLMs, chat, and error handling
- **[Tools](https://helios-engine.vercel.app/tools/using_tools.html)** - Using and creating tools
- **[Forest of Agents](https://helios-engine.vercel.app/forest_of_agents/introduction.html)** - Multi-agent systems
- **[RAG System](https://helios-engine.vercel.app/rag/introduction.html)** - Retrieval-Augmented Generation
- **[Examples](https://helios-engine.vercel.app/examples/overview.html)** - Code examples and use cases

### Local Documentation
- **[ Getting Started](docs/GETTING_STARTED.md)** - Comprehensive guide: installation, configuration, first agent, tools, and CLI
- **[ Tools Guide](docs/TOOLS.md)** - Built-in tools, custom tool creation, and Tool Builder
- **[ Forest of Agents](docs/FOREST.md)** - Multi-agent systems, coordination, and communication
- **[ RAG System](docs/RAG.md)** - Retrieval-Augmented Generation with vector stores
- **[ API Reference](docs/API.md)** - Complete API documentation
- **[ Configuration](docs/CONFIGURATION.md)** - Configuration options and local inference setup
- **[ Using as Crate](docs/USING_AS_CRATE.md)** - Library usage guide

 **[Full Documentation Index](docs/README.md)** - Complete navigation and updated structure

## Quick Start

### Version 0.4.4

#### Install CLI Tool
```bash
# Install without local model support (lighter, faster install)
cargo install helios-engine

# Install with local model support (enables offline mode with llama-cpp-2)
cargo install helios-engine --features local
```

#### Basic Usage
```bash
# Initialize configuration
helios-engine init

# Start interactive chat
helios-engine chat

# Ask a quick question
helios-engine ask "What is Rust?"
```

#### As a Library Crate
Add to your `Cargo.toml`:
```toml
[dependencies]
helios-engine = "0.4.4"
tokio = { version = "1.35", features = ["full"] }
```

For local model support:
```toml
[dependencies]
helios-engine = { version = "0.4.4", features = ["local"] }
tokio = { version = "1.35", features = ["full"] }
```

### Simple Example

```rust
use helios_engine::{Agent, Config, CalculatorTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    let mut agent = Agent::builder("MyAssistant")
        .config(config)
        .system_prompt("You are a helpful AI assistant.")
        .tool(Box::new(CalculatorTool))
        .react()  // Enable ReAct mode for reasoning before acting!
        .build()
        .await?;

    let response = agent.chat("What is 15 * 8?").await?;
    println!("{}", response);

    Ok(())
}
```

See **[📖 Getting Started Guide](docs/GETTING_STARTED.md)** or visit the **[Official Book](https://helios-engine.vercel.app/)** for detailed examples and comprehensive tutorials!

##  Use Cases

- **Chatbots & Virtual Assistants**: Build conversational AI with tool access and memory
- **Multi-Agent Systems**: Coordinate multiple specialized agents for complex workflows
- **Data Analysis**: Agents that can read files, process data, and generate reports
- **Web Automation**: Scrape websites, make API calls, and process responses
- **Knowledge Management**: Build RAG systems for semantic search and Q&A
- **API Services**: Expose your agents via OpenAI-compatible HTTP endpoints
- **Local AI**: Run models completely offline for privacy and security

##  Built-in Tools (16+)

Helios Engine includes a comprehensive suite of production-ready tools:

- **File Management**: Read, write, edit, and search files
- **Web & API**: Web scraping, HTTP requests
- **System Utilities**: Shell commands, system information
- **Data Processing**: JSON parsing, text manipulation, timestamps
- **Communication**: Agent-to-agent messaging
- **Knowledge**: RAG tool for semantic search and retrieval

Learn more in the [Tools Guide](https://helios-engine.vercel.app/tools/using_tools.html).

##  Project Structure

```
helios-engine/
├── src/                    # Source code
├── examples/               # Example applications
├── docs/                   # Documentation
├── book/                   # mdBook source (deployed to vercel)
├── tests/                  # Integration tests
├── Cargo.toml             # Project configuration
└── README.md              # This file
```

##  Contributing

We welcome contributions! See our **[Contributing Guide](https://helios-engine.vercel.app/contributing/how_to_contribute.html)** for details on:
- Development setup
- Code standards
- Documentation guidelines
- Testing procedures

##  Links

- **[Official Website & Book](https://helios-engine.vercel.app/)** - Complete documentation and guides
- **[Crates.io](https://crates.io/crates/helios-engine)** - Package registry
- **[API Documentation](https://docs.rs/helios-engine)** - API reference
- **[GitHub Repository](https://github.com/Ammar-Alnagar/helios-engine)** - Source code
- **[Examples](examples/)** - Code examples

##  License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<p align="center">
  Made with love in Rust by <a href="https://github.com/Ammar-Alnagar">Ammar Alnagar</a>
</p>
