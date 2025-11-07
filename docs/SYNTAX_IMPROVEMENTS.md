# Syntax Improvements - Quick Reference

This document highlights the new, improved syntax introduced in Helios Engine v0.4.3+.

## 🎯 Overview

The new syntax provides a cleaner, more ergonomic way to add multiple tools and agents while maintaining full backward compatibility.

---

## 🔧 Tools Syntax

### Adding Multiple Tools

#### Old Syntax (Still Supported)
```rust
let agent = Agent::builder("MyAgent")
    .config(config)
    .tool(Box::new(CalculatorTool))
    .tool(Box::new(EchoTool))
    .tool(Box::new(FileSearchTool))
    .tool(Box::new(FileReadTool))
    .tool(Box::new(FileWriteTool))
    .build()
    .await?;
```

#### New Syntax (Recommended) ✨
```rust
let agent = Agent::builder("MyAgent")
    .config(config)
    .tools(vec![
        Box::new(CalculatorTool),
        Box::new(EchoTool),
        Box::new(FileSearchTool),
        Box::new(FileReadTool),
        Box::new(FileWriteTool),
    ])
    .build()
    .await?;
```

### Benefits
- ✅ Cleaner and more readable
- ✅ Easier to organize tools into groups
- ✅ Less repetitive code
- ✅ Can combine with individual `.tool()` calls

### Advanced: Organizing Tools

```rust
// Group related tools
let file_tools = vec![
    Box::new(FileSearchTool),
    Box::new(FileReadTool),
    Box::new(FileWriteTool),
    Box::new(FileEditTool),
];

let utility_tools = vec![
    Box::new(CalculatorTool),
    Box::new(EchoTool),
];

let agent = Agent::builder("PowerAgent")
    .config(config)
    .tools(file_tools)      // Add file tools
    .tools(utility_tools)   // Add utility tools
    .build()
    .await?;
```

### Mixing Old and New Syntax

You can mix both styles if needed:

```rust
let agent = Agent::builder("MixedAgent")
    .config(config)
    .tools(vec![
        Box::new(CalculatorTool),
        Box::new(EchoTool),
    ])
    .tool(Box::new(CustomTool))  // Add one more individually
    .build()
    .await?;
```

---

## 🌲 Forest of Agents Syntax

### Adding Multiple Agents

#### Old Syntax (Still Supported)
```rust
let forest = ForestBuilder::new()
    .config(config)
    .agent(
        "coordinator".to_string(),
        Agent::builder("coordinator")
            .system_prompt("You coordinate tasks.")
    )
    .agent(
        "worker1".to_string(),
        Agent::builder("worker1")
            .system_prompt("You process data.")
    )
    .agent(
        "worker2".to_string(),
        Agent::builder("worker2")
            .system_prompt("You generate reports.")
    )
    .build()
    .await?;
```

#### New Syntax (Recommended) ✨
```rust
let forest = ForestBuilder::new()
    .config(config)
    .agents(vec![
        (
            "coordinator".to_string(),
            Agent::builder("coordinator")
                .system_prompt("You coordinate tasks.")
        ),
        (
            "worker1".to_string(),
            Agent::builder("worker1")
                .system_prompt("You process data.")
        ),
        (
            "worker2".to_string(),
            Agent::builder("worker2")
                .system_prompt("You generate reports.")
        ),
    ])
    .build()
    .await?;
```

### Benefits
- ✅ See entire agent hierarchy at a glance
- ✅ Easier to understand agent relationships
- ✅ Better for large multi-agent systems
- ✅ More consistent with Rust idioms

### Advanced: Organizing Agents by Role

```rust
// Define agent teams
let leadership = vec![
    ("ceo".to_string(), Agent::builder("ceo")
        .system_prompt("You make strategic decisions.")),
    ("manager".to_string(), Agent::builder("manager")
        .system_prompt("You coordinate teams.")),
];

let workers = vec![
    ("developer".to_string(), Agent::builder("developer")
        .system_prompt("You write code.")),
    ("tester".to_string(), Agent::builder("tester")
        .system_prompt("You test code.")),
    ("documenter".to_string(), Agent::builder("documenter")
        .system_prompt("You write docs.")),
];

let forest = ForestBuilder::new()
    .config(config)
    .agents(leadership)
    .agents(workers)
    .max_iterations(25)
    .build()
    .await?;
```

### Mixing Old and New Syntax

You can mix both styles:

```rust
let forest = ForestBuilder::new()
    .config(config)
    .agents(vec![
        ("coordinator".to_string(), Agent::builder("coordinator")),
        ("worker1".to_string(), Agent::builder("worker1")),
    ])
    .agent("worker2".to_string(), Agent::builder("worker2"))  // Add one more
    .build()
    .await?;
```

---

## 🔄 Migration Guide

### No Migration Required!

Both old and new syntax work perfectly. You can:

1. **Keep using old syntax** - No changes needed
2. **Gradually adopt new syntax** - Update code as you work on it
3. **Mix both styles** - Use what feels right for each situation

### When to Use Each Style

**Use `.tools(vec![...])` when:**
- Adding 3+ tools at once
- Tools are logically grouped
- You want cleaner, more readable code

**Use `.tool()` when:**
- Adding a single tool
- Conditionally adding tools
- Mixing with `.tools()` calls

**Use `.agents(vec![...])` when:**
- Creating forests with 3+ agents
- Agents represent a team or hierarchy
- You want to see the full structure

**Use `.agent()` when:**
- Adding a single agent
- Conditionally adding agents
- Building forests incrementally

---

## 📊 Comparison

### Code Density

```rust
// Old: 5 lines for 5 tools
.tool(Box::new(Tool1))
.tool(Box::new(Tool2))
.tool(Box::new(Tool3))
.tool(Box::new(Tool4))
.tool(Box::new(Tool5))

// New: 5 tools in cleaner format
.tools(vec![
    Box::new(Tool1),
    Box::new(Tool2),
    Box::new(Tool3),
    Box::new(Tool4),
    Box::new(Tool5),
])
```

### Readability

The new syntax makes it immediately clear that you're adding a *collection* of related items rather than individual unrelated items.

### Consistency with Rust

The new syntax aligns better with common Rust patterns where collections are passed as `Vec` parameters.

---

## 💡 Pro Tips

### 1. Create Tool Constants

```rust
const BASIC_TOOLS: &[fn() -> Box<dyn Tool>] = &[
    || Box::new(CalculatorTool),
    || Box::new(EchoTool),
];

let agent = Agent::builder("MyAgent")
    .tools(BASIC_TOOLS.iter().map(|f| f()).collect())
    .build()
    .await?;
```

### 2. Conditional Tool Sets

```rust
let mut tools = vec![
    Box::new(CalculatorTool),
];

if enable_file_ops {
    tools.extend(vec![
        Box::new(FileReadTool),
        Box::new(FileWriteTool),
    ]);
}

let agent = Agent::builder("MyAgent")
    .tools(tools)
    .build()
    .await?;
```

### 3. Agent Templates

```rust
fn create_worker_agent(name: &str, specialty: &str) -> (String, AgentBuilder) {
    (
        name.to_string(),
        Agent::builder(name)
            .system_prompt(&format!("You are a {} specialist.", specialty))
    )
}

let forest = ForestBuilder::new()
    .agents(vec![
        create_worker_agent("dev", "development"),
        create_worker_agent("qa", "quality assurance"),
        create_worker_agent("docs", "documentation"),
    ])
    .build()
    .await?;
```

---

## 📚 See Also

- [Getting Started Guide](GETTING_STARTED.md) - Full tutorial
- [Tools Guide](TOOLS.md) - Complete tools documentation
- [Forest Guide](FOREST.md) - Multi-agent systems
- [API Reference](API.md) - Complete API docs

---

## 🎉 Examples

Check out these examples that use the new syntax:

- `examples/agent_with_tools.rs` - Tools syntax
- `examples/forest_of_agents.rs` - Forest syntax

---

**Questions?** Open an issue on GitHub or check the [documentation](README.md)!
