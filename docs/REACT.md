# 🧠 ReAct: Reasoning and Acting

## Overview

ReAct (Reasoning and Acting) is a powerful feature in Helios Engine that enables agents to think through problems systematically before taking actions. This pattern improves decision-making, makes the agent's thought process transparent, and leads to better outcomes on complex tasks.

## Table of Contents

- [Quick Start](#quick-start)
- [How It Works](#how-it-works)
- [When to Use ReAct](#when-to-use-react)
- [Examples](#examples)
- [Best Practices](#best-practices)
- [API Reference](#api-reference)

## Quick Start

Enabling ReAct mode is as simple as adding `.react()` to your agent builder:

```rust
use helios_engine::{Agent, Config, CalculatorTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let mut agent = Agent::builder("ReActAgent")
        .config(config)
        .tool(Box::new(CalculatorTool))
        .react()  // ✨ That's it!
        .build()
        .await?;
    
    let response = agent.chat("Calculate (25 * 4) + (100 / 5)").await?;
    println!("{}", response);
    
    Ok(())
}
```

## How It Works

The ReAct pattern follows a two-phase approach:

### 1. Reasoning Phase 💭

When you send a message to a ReAct-enabled agent, it first:

1. **Analyzes** the user's request
2. **Identifies** what information or tools are needed
3. **Creates** a step-by-step plan
4. **Documents** its reasoning process

### 2. Action Phase ⚡

After reasoning, the agent:

1. **Executes** the planned steps
2. **Uses tools** as needed
3. **Provides** the final response

### Example Flow

```
User Query: "Calculate (25 * 4) + (100 / 5)"

💭 Reasoning Phase:
   - User wants to perform arithmetic
   - Two operations needed: multiplication and division
   - Plan: Calculate 25*4, then 100/5, then add results
   
⚡ Action Phase:
   - Tool: calculator(25 * 4) → 100
   - Tool: calculator(100 / 5) → 20
   - Tool: calculator(100 + 20) → 120
   
Response: "The result is 120"
```

## When to Use ReAct

### ✅ Use ReAct When:

- **Complex Tasks**: Multi-step problems requiring coordination
- **Planning Needed**: Tasks that benefit from upfront thinking
- **Debugging**: When you want to see the agent's thought process
- **Critical Operations**: When accuracy is more important than speed
- **Learning**: Understanding how agents approach problems

### ❌ Don't Use ReAct When:

- **Simple Queries**: "What's 2+2?" doesn't need reasoning overhead
- **Speed Critical**: Real-time applications where latency matters
- **No Tools**: ReAct is designed for tool-using agents
- **Streaming Only**: If you only care about final output

## Examples

### Example 1: Mathematical Problem

```rust
use helios_engine::{Agent, Config, CalculatorTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let mut agent = Agent::builder("MathAgent")
        .config(config)
        .system_prompt("You are a math tutor who thinks step by step.")
        .tool(Box::new(CalculatorTool))
        .react()
        .build()
        .await?;
    
    let response = agent.chat(
        "If I have 15 boxes with 12 items each, and I give away 3 boxes, \
         how many items do I have left?"
    ).await?;
    
    println!("{}", response);
    Ok(())
}
```

### Example 2: Multi-Tool Task

```rust
use helios_engine::{Agent, Config, CalculatorTool, FileReadTool, EchoTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let mut agent = Agent::builder("MultiToolAgent")
        .config(config)
        .system_prompt("You are a helpful assistant with multiple capabilities.")
        .tools(vec![
            Box::new(CalculatorTool),
            Box::new(FileReadTool),
            Box::new(EchoTool),
        ])
        .react()  // ReAct helps coordinate multiple tools
        .max_iterations(10)
        .build()
        .await?;
    
    let response = agent.chat(
        "Read the file 'data.txt', count the numbers in it, \
         and calculate their average."
    ).await?;
    
    println!("{}", response);
    Ok(())
}
```

### Example 3: Comparing With and Without ReAct

```rust
use helios_engine::{Agent, Config, CalculatorTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config1 = Config::from_file("config.toml")?;
    let config2 = Config::from_file("config.toml")?;
    
    // Agent without ReAct
    let mut normal_agent = Agent::builder("NormalAgent")
        .config(config1)
        .tool(Box::new(CalculatorTool))
        .build()
        .await?;
    
    // Agent with ReAct
    let mut react_agent = Agent::builder("ReActAgent")
        .config(config2)
        .tool(Box::new(CalculatorTool))
        .react()  // Enable ReAct
        .build()
        .await?;
    
    let query = "Calculate (15 * 3) + (20 * 2)";
    
    println!("Normal Agent:");
    let response1 = normal_agent.chat(query).await?;
    println!("{}\n", response1);
    
    println!("ReAct Agent:");
    let response2 = react_agent.chat(query).await?;
    println!("{}\n", response2);
    
    Ok(())
}
```

## Best Practices

### 1. Use Descriptive System Prompts

Help the agent understand its role and encourage good reasoning:

```rust
.system_prompt(
    "You are a methodical assistant who thinks through problems carefully. \
     Always explain your reasoning before taking action."
)
```

### 2. Combine with Appropriate Tools

ReAct works best when agents have relevant tools:

```rust
.tools(vec![
    Box::new(CalculatorTool),
    Box::new(FileReadTool),
    Box::new(WebScraperTool),
])
.react()
```

### 3. Set Reasonable Iteration Limits

Complex tasks may need more iterations:

```rust
.max_iterations(15)  // Allow more steps for complex reasoning
.react()
```

### 4. Monitor Reasoning Output

The reasoning is printed to stdout, making it easy to debug:

```
💭 ReAct Reasoning:
Let me analyze this step by step...
```

### 5. Use for Appropriate Tasks

Reserve ReAct for tasks that genuinely benefit from reasoning:

```rust
// Good use case
agent.chat("Analyze this data and provide insights").await?;

// Overkill
agent.chat("Hello!").await?;
```

## API Reference

### Builder Method

```rust
pub fn react(self) -> Self
```

Enables ReAct mode for the agent. This method can be chained anywhere in the builder pattern.

**Example:**

```rust
let agent = Agent::builder("MyAgent")
    .config(config)
    .react()  // Can be placed anywhere in the chain
    .tool(Box::new(CalculatorTool))
    .build()
    .await?;
```

### Behavior Changes

When ReAct mode is enabled:

1. **Before tool execution**: Agent generates reasoning
2. **Reasoning prompt**: Internal prompt asks agent to think step-by-step
3. **Output**: Reasoning is printed with `💭 ReAct Reasoning:` prefix
4. **Session**: Reasoning is stored in chat history for context

### Internal Implementation

The agent uses a specialized reasoning prompt:

```
Before taking any action, think through this step by step:

1. What is the user asking for?
2. What information or tools do I need to answer this?
3. What is my plan to solve this problem?

Provide your reasoning in a clear, structured way.
```

## Performance Considerations

### Latency

ReAct mode adds one extra LLM call before the main execution:

- **Without ReAct**: 1 LLM call + tool executions
- **With ReAct**: 2 LLM calls + tool executions

**Impact**: ~1-2 seconds additional latency depending on model

### Token Usage

Additional tokens are consumed for:
- Reasoning prompt (~50 tokens)
- Reasoning response (~100-300 tokens)
- Storing reasoning in context (~100-300 tokens)

**Impact**: ~250-650 additional tokens per query

### When Performance Matters

For latency-sensitive applications, consider:

```rust
// Option 1: Disable ReAct for simple queries
if query.len() < 20 {
    normal_agent.chat(query).await?
} else {
    react_agent.chat(query).await?
}

// Option 2: Use separate agents
let quick_agent = Agent::builder("Quick").config(config).build().await?;
let thinking_agent = Agent::builder("Thinker").config(config).react().build().await?;
```

## Testing

See `tests/react_tests.rs` for comprehensive test examples:

```bash
# Run ReAct-specific tests
cargo test --test react_tests

# Run specific test
cargo test test_react_agent_creation
```

## Troubleshooting

### Reasoning Not Showing

**Problem**: No reasoning output visible

**Solution**: Reasoning is only generated when:
- ReAct mode is enabled (`.react()`)
- Agent has tools registered
- Check stdout for `💭 ReAct Reasoning:` prefix

### Too Much Overhead

**Problem**: ReAct adds too much latency

**Solution**: 
- Use ReAct selectively for complex tasks only
- Consider disabling for simple queries
- Adjust `max_tokens` in config to limit reasoning length

### Reasoning Quality

**Problem**: Poor quality reasoning

**Solution**:
- Improve system prompt to encourage better thinking
- Use more capable models (e.g., GPT-4 instead of GPT-3.5)
- Provide examples of good reasoning in system prompt

## Examples Directory

Complete working examples available:

- `examples/react_agent.rs` - Full ReAct demonstration
- `examples/agent_with_tools.rs` - Compare with/without ReAct

Run examples:

```bash
cargo run --example react_agent
```

## Further Reading

- [FEATURES.md](FEATURES.md) - Overview of all features
- [TOOLS.md](TOOLS.md) - Complete tools guide
- [API.md](API.md) - Full API reference

---

**Questions or Issues?** Open an issue on [GitHub](https://github.com/Ammar-Alnagar/Helios-Engine)!
