# ReAct (Reasoning and Acting)

ReAct is a powerful feature in Helios Engine that enables agents to reason about tasks before taking actions. This pattern leads to more thoughtful, systematic problem-solving and makes the agent's decision-making process transparent.

## What is ReAct?

ReAct (Reasoning and Acting) is a pattern where the agent follows a two-phase approach:

1. **💭 Reasoning Phase**: The agent analyzes the task, identifies what's needed, and creates a plan
2. **⚡ Action Phase**: The agent executes the plan using available tools

This separation helps agents handle complex, multi-step tasks more effectively and provides visibility into their thinking process.

## Enabling ReAct Mode

Enabling ReAct is incredibly simple - just add `.react()` to your agent builder:

```rust
use helios_engine::{Agent, Config, CalculatorTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let mut agent = Agent::builder("ReActAgent")
        .config(config)
        .tool(Box::new(CalculatorTool))
        .react()  // ✨ Enable ReAct mode
        .build()
        .await?;
    
    let response = agent.chat("Calculate (25 * 4) + (100 / 5)").await?;
    println!("{}", response);
    
    Ok(())
}
```

## How It Works

When you send a message to a ReAct-enabled agent, here's what happens:

```
User Query: "Calculate (25 * 4) + (100 / 5)"

💭 Reasoning Phase:
   Agent thinks: "I need to:
   1. Calculate 25 * 4 = 100
   2. Calculate 100 / 5 = 20
   3. Add the results: 100 + 20 = 120"
   
⚡ Action Phase:
   - Uses calculator tool: 25 * 4 → 100
   - Uses calculator tool: 100 / 5 → 20
   - Uses calculator tool: 100 + 20 → 120
   
Response: "The result is 120"
```

The reasoning is displayed with a `💭 ReAct Reasoning:` prefix, making it easy to follow the agent's thought process.

## Custom Reasoning Prompts

For domain-specific tasks, you can customize the reasoning prompt:

```rust
use helios_engine::{Agent, Config, CalculatorTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let math_prompt = r#"As a mathematical problem solver:
1. Identify the mathematical operations needed
2. Break down complex calculations into steps
3. Determine the order of operations (PEMDAS)
4. Plan which calculator functions to use
5. Verify the logic of your approach

Provide clear mathematical reasoning."#;
    
    let mut agent = Agent::builder("MathExpert")
        .config(config)
        .system_prompt("You are a mathematics expert.")
        .tool(Box::new(CalculatorTool))
        .react_with_prompt(math_prompt)  // 🎯 Custom reasoning
        .build()
        .await?;
    
    let response = agent.chat("Calculate ((15 * 8) + (20 * 3)) / 2").await?;
    println!("{}", response);
    
    Ok(())
}
```

## When to Use ReAct

###  Use ReAct When:

- **Complex Multi-Step Tasks**: Tasks that require planning and coordination
- **Debugging**: When you want to see how the agent approaches problems
- **Critical Operations**: When accuracy is more important than speed
- **Learning**: Understanding agent behavior and decision-making
- **Domain-Specific Tasks**: With custom prompts for specialized reasoning

### ❌ Don't Use ReAct When:

- **Simple Queries**: Straightforward tasks where reasoning adds unnecessary overhead
- **Speed Critical**: Applications where latency is paramount
- **No Tools Available**: ReAct is designed for tool-using agents
- **High-Volume Operations**: When the extra LLM call impacts throughput

## Examples

### Example 1: Basic ReAct Agent

```rust
use helios_engine::{Agent, Config, CalculatorTool, EchoTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let mut agent = Agent::builder("Assistant")
        .config(config)
        .tools(vec![
            Box::new(CalculatorTool),
            Box::new(EchoTool),
        ])
        .react()
        .build()
        .await?;
    
    // Multi-step task
    let response = agent
        .chat("Calculate 15 * 7, then echo the result")
        .await?;
    println!("{}", response);
    
    Ok(())
}
```

### Example 2: Domain-Specific Reasoning

```rust
use helios_engine::{Agent, Config, FileReadTool, CalculatorTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let data_analysis_prompt = r#"As a data analyst:
1. UNDERSTAND: What data am I working with?
2. EXTRACT: What information do I need?
3. PROCESS: What calculations are required?
4. TOOLS: Which tools should I use?
5. OUTPUT: How should I present the result?

Think through the data pipeline systematically."#;
    
    let mut analyst = Agent::builder("DataAnalyst")
        .config(config)
        .system_prompt("You are a data analysis expert.")
        .tools(vec![
            Box::new(FileReadTool),
            Box::new(CalculatorTool),
        ])
        .react_with_prompt(data_analysis_prompt)
        .build()
        .await?;
    
    let response = analyst
        .chat("Analyze the numbers: 10, 20, 30, 40, 50. Calculate their average.")
        .await?;
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
    
    // Standard agent
    let mut standard = Agent::builder("Standard")
        .config(config1)
        .tool(Box::new(CalculatorTool))
        .build()
        .await?;
    
    // ReAct agent
    let mut react = Agent::builder("ReAct")
        .config(config2)
        .tool(Box::new(CalculatorTool))
        .react()
        .build()
        .await?;
    
    let query = "Calculate (15 * 3) + (20 * 2)";
    
    println!("Standard agent:");
    let r1 = standard.chat(query).await?;
    println!("{}\n", r1);
    
    println!("ReAct agent:");
    let r2 = react.chat(query).await?;
    println!("{}\n", r2);
    
    Ok(())
}
```

## Builder Methods

### `.react()`

Enables ReAct mode with the default reasoning prompt.

```rust
let agent = Agent::builder("MyAgent")
    .config(config)
    .react()
    .build()
    .await?;
```

### `.react_with_prompt(prompt)`

Enables ReAct mode with a custom reasoning prompt.

```rust
let custom_prompt = "Think step by step about this problem...";

let agent = Agent::builder("MyAgent")
    .config(config)
    .react_with_prompt(custom_prompt)
    .build()
    .await?;
```

Both methods can be placed anywhere in the builder chain:

```rust
// Before tools
let agent = Agent::builder("Agent")
    .config(config)
    .react()
    .tool(Box::new(CalculatorTool))
    .build()
    .await?;

// After tools
let agent = Agent::builder("Agent")
    .config(config)
    .tool(Box::new(CalculatorTool))
    .react()
    .build()
    .await?;
```

## Performance Considerations

### Latency

ReAct adds one extra LLM call for reasoning:

- **Without ReAct**: 1 LLM call + tool executions
- **With ReAct**: 2 LLM calls + tool executions

**Impact**: Approximately 1-2 seconds additional latency (varies by model)

### Token Usage

Additional tokens are consumed for:
- Reasoning prompt: ~50 tokens
- Reasoning response: ~100-300 tokens
- Context storage: ~100-300 tokens

**Impact**: ~250-650 additional tokens per query

### Optimization Tips

For applications where performance matters:

```rust
// Use ReAct selectively
if query_is_complex {
    react_agent.chat(query).await?
} else {
    standard_agent.chat(query).await?
}

// Or create specialized agents
let quick_agent = Agent::builder("Quick")
    .config(config)
    .build()
    .await?;

let thinking_agent = Agent::builder("Thinker")
    .config(config)
    .react()
    .build()
    .await?;
```

## Benefits

### 1. Better Accuracy
Thinking before acting reduces errors and improves decision quality.

### 2. Transparency
See exactly how the agent approaches problems, making debugging easier.

### 3. Complex Task Handling
Multi-step problems are handled more systematically with clear planning.

### 4. Explainability
Understand agent reasoning for compliance, auditing, or learning purposes.

### 5. Domain Adaptation
Custom prompts tailor reasoning to specific domains or tasks.

## Best Practices

### 1. Use Descriptive System Prompts

```rust
.system_prompt("You are a methodical assistant who thinks through problems carefully.")
```

### 2. Combine with Appropriate Tools

```rust
.tools(vec![
    Box::new(CalculatorTool),
    Box::new(FileReadTool),
    Box::new(JsonParserTool),
])
.react()
```

### 3. Set Reasonable Iteration Limits

```rust
.max_iterations(15)  // Allow enough steps for complex reasoning
.react()
```

### 4. Monitor Reasoning Output

Watch the `💭 ReAct Reasoning:` output to understand agent behavior and optimize prompts.

### 5. Use Custom Prompts for Specific Domains

Tailor the reasoning prompt to match your use case (mathematics, data analysis, planning, etc.).

## Troubleshooting

### Reasoning Not Showing

**Problem**: No reasoning output visible

**Solution**: 
- Ensure `.react()` or `.react_with_prompt()` is called
- Verify the agent has tools registered
- Check stdout for `💭 ReAct Reasoning:` prefix

### Too Much Overhead

**Problem**: ReAct adds too much latency

**Solution**:
- Use ReAct selectively for complex tasks only
- Consider disabling for simple queries
- Use faster models for reasoning phase

### Poor Reasoning Quality

**Problem**: Agent reasoning is unclear or unhelpful

**Solution**:
- Improve the system prompt to encourage better thinking
- Use more capable models (e.g., GPT-4 vs GPT-3.5)
- Create custom reasoning prompts with examples
- Adjust the prompt structure for your specific domain

## Next Steps

- Check out the [examples directory](../examples/overview.md) for complete working examples
- See [react_agent.rs](https://github.com/Ammar-Alnagar/Helios-Engine/blob/main/examples/react_agent.rs) for a basic demo
- See [react_custom_prompt.rs](https://github.com/Ammar-Alnagar/Helios-Engine/blob/main/examples/react_custom_prompt.rs) for domain-specific examples
- Read the [Tools documentation](../tools/using_tools.md) to learn about available tools

## Summary

ReAct mode enables agents to think before acting, leading to:
- 🎯 More accurate results
- 👁️ Transparent decision-making
- 🧩 Better handling of complex tasks
- 🔧 Easier debugging and optimization

Simply add `.react()` to your agent builder to enable this powerful feature!
