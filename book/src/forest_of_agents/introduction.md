# Introduction to the Forest of Agents

The Forest of Agents is a multi-agent system where multiple AI agents collaborate to solve complex tasks. Each agent can have specialized roles, tools, and prompts, enabling sophisticated workflows.

## Key Concepts

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

### Adding Multiple Agents at Once

Instead of chaining multiple `.agent()` calls, you can use the `.agents()` method:

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
