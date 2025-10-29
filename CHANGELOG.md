# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-01-XX

### Added

#### Core Features
- Agent system with builder pattern for flexible configuration
- LLM client supporting OpenAI and compatible APIs
- Tool registry for extensible functionality
- Chat session management with conversation history
- TOML-based configuration system
- Comprehensive error handling with custom error types

#### Built-in Tools
- `CalculatorTool` - Perform basic arithmetic operations
- `EchoTool` - Echo messages back for testing

#### Documentation
- Comprehensive README with architecture diagrams
- Mermaid diagrams for system visualization
- ARCHITECTURE.md with detailed design documentation
- Multiple example programs
- API documentation with rustdoc

#### Examples
- `basic_chat.rs` - Simple chat interaction
- `agent_with_tools.rs` - Tool usage demonstration
- `custom_tool.rs` - Custom tool implementation guide
- `multiple_agents.rs` - Multiple agent coordination

#### Infrastructure
- Cargo workspace setup
- CI/CD ready structure
- Example configuration file
- MIT License
- Contributing guidelines

### Features

- **Async/Await**: Full async support with Tokio
- **Type Safety**: Leverages Rust's type system
- **Extensible**: Easy to add custom tools and providers
- **Interactive**: Command-line interface for testing
- **Modular**: Clean separation of concerns

### Technical Details

- Rust Edition 2021
- Tokio for async runtime
- Reqwest for HTTP client
- Serde for serialization
- TOML for configuration
- Tracing for logging

### Known Limitations

- No streaming support yet
- Basic expression evaluator in calculator
- No conversation persistence
- No rate limiting built-in

### Dependencies

- tokio = "1.35"
- serde = "1.0"
- serde_json = "1.0"
- reqwest = "0.11"
- toml = "0.8"
- anyhow = "1.0"
- thiserror = "1.0"
- async-trait = "0.1"
- tracing = "0.1"
- tracing-subscriber = "0.3"

## [Unreleased]

### Planned Features

- Streaming response support
- Tool result caching
- Conversation persistence
- More built-in tools
- Plugin system
- Multi-modal support
- Agent collaboration
- Observability and metrics

---

[0.1.0]: https://github.com/yourusername/helios/releases/tag/v0.1.0
