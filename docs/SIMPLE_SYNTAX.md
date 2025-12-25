# Helios Engine - Simplest Possible Syntax Guide

## Philosophy

**Helios is designed for SIMPLICITY.** Every feature has:
- ✅ A full, explicit API for control
- ✅ Short aliases for common operations
- ✅ Convenience methods for the fastest path

This guide shows you the **SHORTEST** way to do everything.

---

## The Absolute Fastest Start

### Before You Read Anything Else

```rust
use helios_engine::Agent;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let mut agent = Agent::quick("Bot").await?;
    let response = agent.ask("Hello!").await?;
    println!("{}", response);
    Ok(())
}
```

**That's it.** You have a working agent.

---

## Quick Reference - Shortest Syntax

### Agents

| What | Shortest | Normal |
|------|----------|--------|
| Create quick agent | `Agent::quick("name").await?` | `Agent::builder("name").auto_config().build().await?` |
| Chat | `agent.ask("question")` | `agent.chat("question")` |
| With prompt | `.prompt("text")` | `.system_prompt("text")` |
| Add tool | `.with_tool(tool)` | `.tool(tool)` |
| Add multiple tools | `.with_tools(vec![...])` | `.tools(vec![...])` |

### Config

| What | Shortest | Normal |
|------|----------|--------|
| Model | `Config::builder().m("gpt-4")` | `Config::builder().model("gpt-4")` |
| API Key | `.key("key")` | `.api_key("key")` |
| Base URL | `.url("url")` | `.base_url("url")` |
| Temperature | `.temp(0.8)` | `.temperature(0.8)` |
| Max Tokens | `.tokens(1024)` | `.max_tokens(1024)` |

### Messages

| What | Shortest | Normal |
|------|----------|--------|
| System | `ChatMessage::sys()` | `ChatMessage::system()` |
| User | `ChatMessage::msg()` | `ChatMessage::user()` |
| Assistant | `ChatMessage::reply()` | `ChatMessage::assistant()` |

### AutoForest

| What | Shortest | Normal |
|------|----------|--------|
| Execute task | `forest.run("task")` | `forest.execute_task("task")` |
| Execute task | `forest.do_task("task")` | `forest.execute_task("task")` |

### ForestBuilder

| What | Shortest | Normal |
|------|----------|--------|
| Add agents | `.agents(vec![...])` | `.add_agents(vec![...])` |
| Add single agent | `.agent(name, builder)` | `.add_agent(name, builder)` |

### ChatSession

| What | Shortest | Normal |
|------|----------|--------|
| Add system | `session.add_sys()` | `session.add_message(ChatMessage::system())` |
| Add user | `session.add_msg()` | `session.add_user_message()` |
| Add reply | `session.add_reply()` | `session.add_assistant_message()` |

---

## Real Examples - Getting Shorter

### Example 1: Create and Chat

**Longest (but most explicit):**
```rust
let config = Config::builder()
    .model("gpt-4")
    .api_key("key")
    .temperature(0.7)
    .max_tokens(2048)
    .build();

let mut agent = Agent::builder("Assistant")
    .config(config)
    .system_prompt("You are helpful")
    .build()
    .await?;

let response = agent.chat("Hello").await?;
```

**Medium:**
```rust
let config = Config::builder()
    .m("gpt-4")
    .key("key")
    .build();

let mut agent = Agent::builder("Assistant")
    .config(config)
    .prompt("You are helpful")
    .build()
    .await?;

let response = agent.ask("Hello").await?;
```

**Shortest:**
```rust
let mut agent = Agent::quick("Assistant").await?;
let response = agent.ask("Hello").await?;
```

### Example 2: Agent with Tools

**Longest:**
```rust
let config = Config::new_default();
let mut agent = Agent::builder("Math")
    .config(config)
    .tool(Box::new(CalculatorTool))
    .build()
    .await?;
```

**Shortest:**
```rust
let mut agent = Agent::quick("Math").await?;
agent.with_tool(Box::new(CalculatorTool));
```

### Example 3: Working with Messages

**Longest:**
```rust
let mut session = ChatSession::new();
session.with_system_prompt("You are helpful");
session.add_message(ChatMessage::system("You are helpful"));
session.add_message(ChatMessage::user("Hello"));
session.add_message(ChatMessage::assistant("Hi!"));
```

**Shortest:**
```rust
let mut session = ChatSession::new()
    .with_system_prompt("You are helpful");
session.add_sys("You are helpful");
session.add_msg("Hello");
session.add_reply("Hi!");
```

### Example 4: AutoForest Orchestration

**Longest:**
```rust
let mut forest = AutoForest::new(config)
    .with_tools(vec![Box::new(CalculatorTool)])
    .build()
    .await?;

let result = forest.execute_task("Analyze this data").await?;
```

**Shortest:**
```rust
let mut forest = AutoForest::new(Config::builder().m("gpt-4").build())
    .with_tools(vec![Box::new(CalculatorTool)])
    .build()
    .await?;

let result = forest.run("Analyze this data").await?;
```

---

## Method Chaining for Maximum Simplicity

All builders support fluent chaining for ultra-concise code:

```rust
// Config - all in one line
let config = Config::builder()
    .m("gpt-4")
    .key("key")
    .temp(0.8)
    .tokens(2048)
    .build();

// Agent - all in one line
let mut agent = Agent::builder("Bot")
    .auto_config()
    .prompt("Be helpful")
    .with_tool(Box::new(CalculatorTool))
    .build()
    .await?;

// Immediate use
let answer = agent.ask("What is 5+3?").await?;
```

---

## One-Liners for Common Tasks

### Create and use an agent
```rust
let response = Agent::quick("Bot").await?.ask("Hello").await?;
```

### Create agent with config
```rust
Agent::builder("Bot")
    .config(Config::builder().m("gpt-4").key("key").build())
    .build()
    .await?
```

### Create agent with tools
```rust
Agent::quick("Bot").await?
    .with_tools(vec![Box::new(CalculatorTool)])
```

### Run AutoForest
```rust
AutoForest::new(Config::builder().m("gpt-4").build())
    .with_tools(vec![Box::new(CalculatorTool)])
    .build()
    .await?
    .run("task")
    .await?
```

---

## Finding the Right Method

### "I want to create a config"
- **Simplest**: `Config::new_default()`
- **Customized**: `Config::builder().m("model").key("key").build()`
- **From file**: `Config::from_file("config.toml")?`

### "I want to create an agent"
- **Quickest**: `Agent::quick("name").await?`
- **Customized**: `Agent::builder("name").auto_config().prompt("text").build().await?`
- **Full control**: `Agent::builder("name").config(config).build().await?`

### "I want to chat"
- **Simple**: `agent.ask("question").await?`
- **Detailed**: `agent.chat("question").await?`
- **Custom**: `agent.send_message("question").await?`

### "I want to use tools"
- **Single**: `.with_tool(Box::new(Tool))`
- **Multiple**: `.with_tools(vec![Box::new(T1), Box::new(T2)])`
- **Via builder**: `.tool(Box::new(Tool))`

### "I want AutoForest"
- **Quick**: `forest.run("task").await?`
- **Explicit**: `forest.execute_task("task").await?`
- **Alternative**: `forest.do_task("task").await?`

---

## Mixing Short and Long Syntax

You can mix and match! Use short syntax for common things, long syntax for specific control:

```rust
// Short config
let config = Config::builder()
    .m("gpt-4")
    .key("key")
    .build();

// Detailed agent setup
let mut agent = Agent::builder("Expert")
    .config(config)
    .system_prompt("You are an expert in finance")
    .with_tools(vec![
        Box::new(CalculatorTool),
        Box::new(FileReadTool),
    ])
    .build()
    .await?;

// Quick chat
let answer = agent.ask("What's the ROI?").await?;
```

---

## Best Practices for Simple Code

### ✅ DO

- Use `Agent::quick()` for getting started
- Use `.ask()` for natural conversation
- Use `.m()`, `.key()`, `.temp()` shortcuts in Config
- Chain methods for readability
- Use `auto_config()` when you don't need custom config

### ❌ DON'T

- Use verbose names when short aliases exist
- Create unnecessary intermediate variables
- Write more code than necessary for simple tasks

---

## Advanced: When to Use Long Forms

Use the explicit, longer API when you need:

1. **Specific control** - `set_max_iterations()` vs `max_iterations()`
2. **Clear intent** - `system_prompt()` is clearer than `prompt()`
3. **Documentation** - Full names are self-documenting
4. **Testing** - Explicit names help in test code

---

## Comparison with Other Frameworks

### Helios (Ultra-Simple)
```rust
Agent::quick("Bot").await?.ask("Hi").await?
```

### Alternative (More Verbose)
```rust
let config = Config::new_default();
let mut agent = Agent::builder("Bot").config(config).build().await?;
agent.chat("Hi").await?
```

**Helios wins on simplicity** while keeping full power available.

---

## Summary

Helios philosophy:
- 🎯 **Simple by default** - `Agent::quick()` gets you started instantly
- 🔧 **Powerful when needed** - Full APIs available for advanced users
- 🏗️ **Flexible syntax** - Use short or long forms based on context
- ⚡ **Zero boilerplate** - Do more with less code

**Start simple, scale up as needed.**

For more details on specific features, see:
- [README.md](../README.md) - Main documentation
- [API.md](API.md) - Full API reference
- [examples/](../examples/) - Working code examples
