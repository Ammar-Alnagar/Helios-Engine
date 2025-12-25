# Helios Engine Documentation

Welcome to the Helios Engine documentation! This guide has been reorganized and streamlined for clarity and ease of use.

## Documentation Structure

### Getting Started
- **[GETTING_STARTED.md](GETTING_STARTED.md)** - Complete guide to get up and running quickly
  - Installation
  - Configuration
  - Basic usage examples
  - Building your first agent
  - CLI reference

### Core Features
- **[REACT.md](REACT.md)** - ReAct (Reasoning and Acting) guide
  - How ReAct works
  - When to use it
  - Examples and best practices
  - Performance considerations

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

## Quick Navigation

**New to Helios Engine?**  
→ Start with [GETTING_STARTED.md](GETTING_STARTED.md)

**Want reasoning agents?**
→ See [REACT.md](REACT.md)

**Want to use tools?**  
→ See [TOOLS.md](TOOLS.md)

**Building multi-agent systems?**  
→ See [FOREST.md](FOREST.md)

**Need RAG capabilities?**  
→ See [RAG.md](RAG.md)

**Looking for API details?**  
→ See [API.md](API.md)

## Quick Start

### Absolute Easiest Way (3 lines!)

```rust
use helios_engine::Agent;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let mut agent = Agent::quick("MyBot").await?;
    let response = agent.ask("What is 2+2?").await?;
    println!("{}", response);
    Ok(())
}
```

### Traditional Builder Approach (With Custom Config)

```rust
use helios_engine::{Agent, Config, CalculatorTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Using shortened syntax for config
    let config = Config::builder()
        .m("gpt-4")
        .key("your-api-key")
        .temp(0.7)
        .build();
    
    let mut agent = Agent::builder("MyAgent")
        .config(config)
        .prompt("You are a helpful math assistant")
        .with_tools(vec![Box::new(CalculatorTool)])
        .build()
        .await?;
    
    let response = agent.ask("What is 15 * 8?").await?;
    println!("{}", response);
    
    Ok(())
}
```

### With Auto Config

```rust
let mut agent = Agent::builder("MyAgent")
    .auto_config()  // Automatically loads config.toml or uses defaults
    .prompt("You are helpful")
    .with_tool(Box::new(CalculatorTool))
    .build()
    .await?;

let response = agent.ask("What is 15 * 8?").await?;
```

## What's New

### ReAct Feature (v0.4.5+)

**Enable reasoning before acting with a simple `.react()` call:**

```rust
let mut agent = Agent::builder("MyAgent")
    .config(config)
    .tool(Box::new(CalculatorTool))
    .react()  // Agent now reasons before acting!
    .build()
    .await?;
```

The agent will now:
1. **Reason** about the task
2. **Plan** the approach
3. **Execute** with tools

See `examples/react_agent.rs` for details!

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

## Documentation Philosophy

This documentation follows these principles:

1. **Consolidation** - Related information is grouped together
2. **Clarity** - Clear examples and explanations
3. **Completeness** - Comprehensive coverage of features
4. **Consistency** - Consistent style and formatting
5. **Currency** - Up-to-date with latest features

## External Resources

- [GitHub Repository](https://github.com/Ammar-Alnagar/Helios-Engine)
- [Crates.io](https://crates.io/crates/helios-engine)
- [API Documentation](https://docs.rs/helios-engine)

## Examples

Check out the `examples/` directory for working code:

- `basic_chat.rs` - Simple chat example
- `agent_with_tools.rs` - Agent with tools (uses new syntax!)
- `react_agent.rs` - ReAct (Reasoning and Acting) example
- `react_custom_prompt.rs` - Custom reasoning prompts for different domains
- `react_comparison.rs` - Compare standard vs ReAct agents
- `react_debugging.rs` - Using ReAct for debugging
- `forest_of_agents.rs` - Multi-agent system (uses new syntax!)
- `forest_with_coordinator.rs` - Coordinator-based planning
- `streaming_chat.rs` - Streaming responses
- `rag_advanced.rs` - RAG implementation
- And many more!

##  Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines.

##  License

See [LICENSE](../LICENSE) for license information.

---

**Questions?** Open an issue on GitHub or check the documentation!
