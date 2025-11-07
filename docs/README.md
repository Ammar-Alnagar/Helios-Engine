# Helios Engine Documentation

Welcome to the Helios Engine documentation! This guide has been reorganized and streamlined for clarity and ease of use.

## 📚 Documentation Structure

### Getting Started
- **[GETTING_STARTED.md](GETTING_STARTED.md)** - Complete guide to get up and running quickly
  - Installation
  - Configuration
  - Basic usage examples
  - Building your first agent
  - CLI reference

### Core Features
- **[TOOLS.md](TOOLS.md)** - Complete tools guide
  - Using built-in tools
  - Creating custom tools
  - Tool builder patterns
  - Best practices
  
- **[FOREST.md](FOREST.md)** - Forest of Agents guide
  - Multi-agent systems
  - Coordinator-based planning
  - Agent communication
  - Advanced patterns

- **[RAG.md](RAG.md)** - Retrieval Augmented Generation
  - Vector databases
  - Document indexing
  - Semantic search

### Reference
- **[API.md](API.md)** - Complete API reference
- **[CONFIGURATION.md](CONFIGURATION.md)** - Configuration options
- **[FEATURES.md](FEATURES.md)** - Feature overview
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture
- **[USING_AS_CRATE.md](USING_AS_CRATE.md)** - Library usage guide

## 🚀 Quick Navigation

**New to Helios Engine?**  
→ Start with [GETTING_STARTED.md](GETTING_STARTED.md)

**Want to use tools?**  
→ See [TOOLS.md](TOOLS.md)

**Building multi-agent systems?**  
→ See [FOREST.md](FOREST.md)

**Need RAG capabilities?**  
→ See [RAG.md](RAG.md)

**Looking for API details?**  
→ See [API.md](API.md)

## 🎯 Quick Start

### Installation
```bash
cargo install helios-engine
```

### First Agent
```rust
use helios_engine::{Agent, Config, CalculatorTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let mut agent = Agent::builder("MyAgent")
        .config(config)
        .tools(vec![Box::new(CalculatorTool)])
        .build()
        .await?;
    
    let response = agent.chat("What is 15 * 8?").await?;
    println!("{}", response);
    
    Ok(())
}
```

## 🆕 What's New

### Improved Syntax (v0.4.3+)

**Multiple tools at once:**
```rust
// Old way
.tool(Box::new(CalculatorTool))
.tool(Box::new(EchoTool))
.tool(Box::new(FileReadTool))

// New way - much cleaner!
.tools(vec![
    Box::new(CalculatorTool),
    Box::new(EchoTool),
    Box::new(FileReadTool),
])
```

**Multiple agents at once:**
```rust
// Old way
.agent("worker1".to_string(), Agent::builder("worker1"))
.agent("worker2".to_string(), Agent::builder("worker2"))

// New way - much cleaner!
.agents(vec![
    ("worker1".to_string(), Agent::builder("worker1")),
    ("worker2".to_string(), Agent::builder("worker2")),
])
```

## 📖 Documentation Philosophy

This documentation follows these principles:

1. **Consolidation** - Related information is grouped together
2. **Clarity** - Clear examples and explanations
3. **Completeness** - Comprehensive coverage of features
4. **Consistency** - Consistent style and formatting
5. **Currency** - Up-to-date with latest features

## 🔗 External Resources

- [GitHub Repository](https://github.com/Ammar-Alnagar/Helios-Engine)
- [Crates.io](https://crates.io/crates/helios-engine)
- [API Documentation](https://docs.rs/helios-engine)

## 💡 Examples

Check out the `examples/` directory for working code:

- `basic_chat.rs` - Simple chat example
- `agent_with_tools.rs` - Agent with tools (uses new syntax!)
- `forest_of_agents.rs` - Multi-agent system (uses new syntax!)
- `forest_with_coordinator.rs` - Coordinator-based planning
- `streaming_chat.rs` - Streaming responses
- `rag_advanced.rs` - RAG implementation
- And many more!

## 🤝 Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines.

## 📝 License

See [LICENSE](../LICENSE) for license information.

---

**Questions?** Open an issue on GitHub or check the documentation!
