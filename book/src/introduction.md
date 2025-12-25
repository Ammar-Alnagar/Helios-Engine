# 🔥 Helios Engine - LLM Agent Framework

<p align="center">
  <img src="Helios_Engine_Logo.png" alt="Helios Engine Logo" width="350"/>
</p>

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
