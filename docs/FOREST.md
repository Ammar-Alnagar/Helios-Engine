# Forest of Agents - Complete Guide

A comprehensive guide to building and using multi-agent systems with Helios Engine.

## Table of Contents
- [Overview](#overview)
- [Basic Usage](#basic-usage)
- [Coordinator-Based Planning](#coordinator-based-planning)
- [Agent Communication](#agent-communication)
- [Advanced Patterns](#advanced-patterns)
- [Best Practices](#best-practices)
- [Examples](#examples)

## Overview

The Forest of Agents is a multi-agent system where multiple AI agents collaborate to solve complex tasks. Each agent can have specialized roles, tools, and prompts, enabling sophisticated workflows.

### Key Concepts

- **Forest**: The container that manages multiple agents
- **Coordinator**: An optional special agent that plans and delegates tasks
- **Worker Agents**: Specialized agents that execute specific tasks
- **Task Planning**: Automatic decomposition of complex tasks into subtasks
- **Agent Communication**: Agents can pass messages and results to each other

## Basic Usage

### Creating a Simple Forest

```rust
use helios_engine::{Agent, Config, ForestBuilder};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    let mut forest = ForestBuilder::new()
        .config(config)
        .agent("worker1".to_string(), 
            Agent::builder("worker1")
                .system_prompt("You are a data analyst."))
        .agent("worker2".to_string(),
            Agent::builder("worker2")
                .system_prompt("You are a report writer."))
        .max_iterations(15)
        .build()
        .await?;
    
    // Execute a task
    let result = forest.execute("Analyze sales data and write a report").await?;
    println!("Result: {}", result);
    
    Ok(())
}
```

### Adding Multiple Agents at Once (New Syntax!)

Instead of chaining multiple `.agent()` calls:

```rust
let mut forest = ForestBuilder::new()
    .config(config)
    .agents(vec![
        ("coordinator".to_string(), Agent::builder("coordinator")
            .system_prompt("You coordinate and plan tasks.")),
        ("researcher".to_string(), Agent::builder("researcher")
            .system_prompt("You research information.")),
        ("analyst".to_string(), Agent::builder("analyst")
            .system_prompt("You analyze data.")),
        ("writer".to_string(), Agent::builder("writer")
            .system_prompt("You write clear documentation.")),
    ])
    .max_iterations(25)
    .build()
    .await?;
```

### Forest Configuration Options

```rust
let forest = ForestBuilder::new()
    .config(config)
    .max_iterations(20)           // Maximum iterations for task execution
    .enable_coordinator_planning() // Enable automatic task planning
    .agents(/* ... */)
    .build()
    .await?;
```

## Coordinator-Based Planning

The coordinator-based planning system enables automatic task decomposition and delegation.

### How It Works

1. **Task Analysis**: The coordinator analyzes the incoming task
2. **Plan Creation**: Creates a structured plan with subtasks
3. **Agent Selection**: Assigns subtasks to appropriate worker agents
4. **Execution**: Worker agents execute their assigned subtasks
5. **Result Aggregation**: The coordinator combines results into a final output

### Enabling Coordinator Planning

```rust
let forest = ForestBuilder::new()
    .config(config)
    .enable_coordinator_planning()
    .coordinator_agent("coordinator".to_string(),
        Agent::builder("coordinator")
            .system_prompt("You are a master coordinator who creates plans."))
    .agents(vec![
        ("researcher".to_string(), Agent::builder("researcher")
            .system_prompt("You research topics thoroughly.")),
        ("coder".to_string(), Agent::builder("coder")
            .system_prompt("You write clean, efficient code.")),
        ("tester".to_string(), Agent::builder("tester")
            .system_prompt("You test code for bugs and issues.")),
    ])
    .max_iterations(30)
    .build()
    .await?;

let result = forest.execute("Research, implement, and test a binary search algorithm").await?;
```

### Plan Structure

The coordinator creates plans in JSON format:

```json
{
  "task": "Research, implement, and test a binary search algorithm",
  "plan": [
    {
      "step": 1,
      "agent": "researcher",
      "action": "Research binary search algorithm and best practices",
      "expected_output": "Detailed explanation and pseudocode"
    },
    {
      "step": 2,
      "agent": "coder",
      "action": "Implement binary search in Rust based on research",
      "expected_output": "Working Rust implementation"
    },
    {
      "step": 3,
      "agent": "tester",
      "action": "Test the implementation with various test cases",
      "expected_output": "Test results and bug report"
    }
  ]
}
```

### Custom Coordinator Prompts

You can customize how the coordinator creates plans:

```rust
let coordinator_prompt = r#"You are an expert project coordinator.

Your role is to:
1. Analyze complex tasks and break them into clear subtasks
2. Assign subtasks to the most appropriate agent
3. Ensure dependencies between tasks are respected
4. Create comprehensive plans in JSON format

Available agents:
- researcher: Gathers information and does analysis
- developer: Writes code and implements features
- reviewer: Reviews code quality and suggests improvements

Always create plans that are specific, measurable, and achievable."#;

let forest = ForestBuilder::new()
    .config(config)
    .enable_coordinator_planning()
    .coordinator_agent("coordinator".to_string(),
        Agent::builder("coordinator")
            .system_prompt(coordinator_prompt))
    .agents(vec![
        ("researcher".to_string(), Agent::builder("researcher")),
        ("developer".to_string(), Agent::builder("developer")),
        ("reviewer".to_string(), Agent::builder("reviewer")),
    ])
    .build()
    .await?;
```

## Agent Communication

### Send Message Tool

Agents can communicate with each other using the `SendMessageTool`:

```rust
use helios_engine::{Agent, SendMessageTool, ForestBuilder};

let forest = ForestBuilder::new()
    .config(config)
    .agent("agent1".to_string(),
        Agent::builder("agent1")
            .tool(Box::new(SendMessageTool::new(forest_handle.clone()))))
    .build()
    .await?;
```

The tool allows agents to send messages like:
```json
{
  "to_agent": "agent2",
  "message": "Please analyze this data: [1, 2, 3, 4, 5]"
}
```

### Accessing Agent Results

```rust
// Get a specific agent's last response
if let Some(agent) = forest.get_agent("researcher") {
    let history = agent.session().get_messages();
    // Process history...
}

// List all agents
let agent_ids = forest.list_agents();
for id in agent_ids {
    println!("Agent: {}", id);
}
```

## Advanced Patterns

### Specialized Tool Sets per Agent

Give each agent different tools based on their role:

```rust
use helios_engine::{CalculatorTool, FileSearchTool, FileReadTool, FileWriteTool};

let forest = ForestBuilder::new()
    .config(config)
    .agents(vec![
        ("analyst".to_string(), 
            Agent::builder("analyst")
                .system_prompt("You analyze data.")
                .tools(vec![
                    Box::new(CalculatorTool),
                    Box::new(FileReadTool),
                ])),
        ("writer".to_string(),
            Agent::builder("writer")
                .system_prompt("You create reports.")
                .tools(vec![
                    Box::new(FileWriteTool),
                    Box::new(FileReadTool),
                ])),
    ])
    .build()
    .await?;
```

### Hierarchical Forest Structure

Create a hierarchy of coordinators and workers:

```rust
// High-level coordinator
let main_forest = ForestBuilder::new()
    .config(config.clone())
    .enable_coordinator_planning()
    .coordinator_agent("main_coordinator".to_string(),
        Agent::builder("main_coordinator")
            .system_prompt("You oversee the entire project."))
    .agents(vec![
        ("dev_team_coordinator".to_string(), Agent::builder("dev_coordinator")),
        ("qa_team_coordinator".to_string(), Agent::builder("qa_coordinator")),
    ])
    .build()
    .await?;

// Separate forests for each team
let dev_forest = ForestBuilder::new()
    .config(config.clone())
    .agents(vec![
        ("frontend_dev".to_string(), Agent::builder("frontend")),
        ("backend_dev".to_string(), Agent::builder("backend")),
    ])
    .build()
    .await?;
```

### Iterative Refinement Pattern

Use forests for iterative improvement:

```rust
let mut forest = ForestBuilder::new()
    .config(config)
    .agents(vec![
        ("creator".to_string(), Agent::builder("creator")
            .system_prompt("You create initial solutions.")),
        ("critic".to_string(), Agent::builder("critic")
            .system_prompt("You critique and suggest improvements.")),
        ("refiner".to_string(), Agent::builder("refiner")
            .system_prompt("You refine based on feedback.")),
    ])
    .max_iterations(10)
    .build()
    .await?;

// Multiple refinement cycles
for cycle in 1..=3 {
    println!("Refinement cycle {}", cycle);
    let result = forest.execute("Improve the solution").await?;
}
```

## Best Practices

### 1. Clear Agent Roles

Give each agent a specific, well-defined role:

✅ **Good:**
```rust
.system_prompt("You are a data analyst specializing in sales metrics. 
                Analyze data and provide statistical insights.")
```

❌ **Avoid:**
```rust
.system_prompt("You are helpful.")
```

### 2. Appropriate Max Iterations

- Simple tasks: 5-10 iterations
- Medium tasks: 10-20 iterations
- Complex multi-agent tasks: 20-40 iterations

```rust
.max_iterations(25)  // Good for most coordinator-based tasks
```

### 3. Task Decomposition

When using coordinator planning, provide clear, decomposable tasks:

✅ **Good:**
```rust
forest.execute("Research Rust error handling, implement examples, 
                and write documentation").await?
```

❌ **Avoid:**
```rust
forest.execute("Do something with Rust").await?
```

### 4. Agent Specialization

Match tools to agent roles:

```rust
let forest = ForestBuilder::new()
    .agents(vec![
        ("file_worker".to_string(), 
            Agent::builder("file_worker")
                .tools(vec![
                    Box::new(FileSearchTool),
                    Box::new(FileReadTool),
                    Box::new(FileWriteTool),
                ])),
        ("calculator".to_string(),
            Agent::builder("calculator")
                .tools(vec![Box::new(CalculatorTool)])),
    ])
    .build()
    .await?;
```

### 5. Error Handling

Always handle forest execution errors:

```rust
match forest.execute(task).await {
    Ok(result) => println!("Success: {}", result),
    Err(e) => eprintln!("Forest execution failed: {}", e),
}
```

## Examples

### Example 1: Software Development Team

```rust
let dev_team = ForestBuilder::new()
    .config(config)
    .enable_coordinator_planning()
    .coordinator_agent("tech_lead".to_string(),
        Agent::builder("tech_lead")
            .system_prompt("You are a technical lead who plans development tasks."))
    .agents(vec![
        ("architect".to_string(), Agent::builder("architect")
            .system_prompt("You design system architecture.")),
        ("developer".to_string(), Agent::builder("developer")
            .system_prompt("You implement features.")),
        ("tester".to_string(), Agent::builder("tester")
            .system_prompt("You write and run tests.")),
        ("documenter".to_string(), Agent::builder("documenter")
            .system_prompt("You write clear documentation.")),
    ])
    .max_iterations(30)
    .build()
    .await?;

let result = dev_team.execute(
    "Design and implement a REST API for user authentication with tests and docs"
).await?;
```

### Example 2: Research and Writing

```rust
let content_team = ForestBuilder::new()
    .config(config)
    .enable_coordinator_planning()
    .coordinator_agent("editor".to_string(),
        Agent::builder("editor")
            .system_prompt("You plan content creation workflows."))
    .agents(vec![
        ("researcher".to_string(), Agent::builder("researcher")
            .system_prompt("You research topics thoroughly.")
            .tools(vec![Box::new(FileSearchTool), Box::new(FileReadTool)])),
        ("writer".to_string(), Agent::builder("writer")
            .system_prompt("You write engaging content.")
            .tools(vec![Box::new(FileWriteTool)])),
        ("fact_checker".to_string(), Agent::builder("fact_checker")
            .system_prompt("You verify facts and citations.")),
    ])
    .max_iterations(25)
    .build()
    .await?;

let article = content_team.execute(
    "Research and write an article about Rust's ownership system"
).await?;
```

### Example 3: Data Pipeline

```rust
let data_pipeline = ForestBuilder::new()
    .config(config)
    .agents(vec![
        ("collector".to_string(), Agent::builder("collector")
            .system_prompt("You collect and validate data.")
            .tools(vec![Box::new(FileReadTool)])),
        ("processor".to_string(), Agent::builder("processor")
            .system_prompt("You process and transform data.")
            .tools(vec![Box::new(CalculatorTool)])),
        ("reporter".to_string(), Agent::builder("reporter")
            .system_prompt("You generate reports.")
            .tools(vec![Box::new(FileWriteTool)])),
    ])
    .max_iterations(20)
    .build()
    .await?;
```

## Troubleshooting

### Common Issues

**Problem**: Forest gets stuck in loops
- **Solution**: Reduce `max_iterations` or improve agent prompts

**Problem**: Agents don't collaborate effectively
- **Solution**: Use coordinator planning or SendMessageTool

**Problem**: Tasks aren't properly decomposed
- **Solution**: Improve coordinator prompt with specific instructions

**Problem**: Agents use wrong tools
- **Solution**: Give each agent only the tools they need

## See Also

- [Getting Started Guide](GETTING_STARTED.md)
- [Tools Documentation](TOOLS.md)
- [API Reference](API.md)
- [Examples Directory](../examples/)

## Running Examples

```bash
# Simple forest demo
cargo run --example forest_simple_demo

# Coordinator-based planning
cargo run --example forest_with_coordinator

# Full forest features
cargo run --example forest_of_agents
```

---

For more advanced usage and API details, see the [API Documentation](API.md).
