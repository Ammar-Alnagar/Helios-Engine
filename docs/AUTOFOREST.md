# AutoForest - Automatic Agent Orchestration

## Overview

AutoForest is an advanced feature in the Helios Engine that automatically orchestrates a forest of specialized agents to complete complex tasks. Instead of manually creating and configuring multiple agents, AutoForest intelligently:

- **Analyzes** the task to understand its complexity and requirements
- **Determines** the optimal number of agents needed
- **Generates** specialized system prompts for each agent
- **Distributes** available tools among agents based on their roles
- **Coordinates** task execution across all agents
- **Aggregates** results into a comprehensive final response

## How It Works

### 1. Task Analysis & Planning

When you submit a task to AutoForest, it creates an orchestrator agent that analyzes the task and generates an `OrchestrationPlan`. This plan includes:

- **Number of agents**: How many specialized agents should be spawned (1-5)
- **Agent roles**: What each agent specializes in
- **System prompts**: Customized instructions for each agent's expertise
- **Tool assignments**: Which tools each agent has access to
- **Task breakdown**: Specific subtasks for each agent

### 2. Dynamic Agent Spawning

Based on the orchestration plan, AutoForest spawns specialized agents with:
- Unique configurations tailored to their role
- Specific system prompts guiding their behavior
- Access to relevant tools for their task

### 3. Parallel Execution

Each agent works on their assigned subtask in parallel, enabling efficient distributed task completion.

### 4. Result Aggregation

Results from all agents are collected and synthesized by the orchestrator into a cohesive final response.

## Architecture

### Core Components

#### `AutoForest`
The main orchestrator struct that manages the forest lifecycle:
```rust
pub struct AutoForest {
    config: Config,
    tools: Vec<Box<dyn Tool>>,
    spawned_agents: Vec<SpawnedAgent>,
    orchestration_plan: Option<OrchestrationPlan>,
    orchestrator_agent: Option<Agent>,
}
```

#### `AgentConfig`
Configuration for each spawned agent:
```rust
pub struct AgentConfig {
    pub name: String,                    // Agent name
    pub system_prompt: String,           // Specialized instructions
    pub tool_indices: Vec<usize>,        // Assigned tools
    pub role: String,                    // Agent's specialty/role
}
```

#### `OrchestrationPlan`
The strategic plan generated for a task:
```rust
pub struct OrchestrationPlan {
    pub task: String,                    // Original task
    pub num_agents: usize,               // Number of agents to spawn
    pub reasoning: String,               // Why this configuration was chosen
    pub agents: Vec<AgentConfig>,        // Configuration for each agent
    pub task_breakdown: HashMap<String, String>, // Subtasks per agent
}
```

#### `SpawnedAgent`
An agent spawned by the orchestrator:
```rust
pub struct SpawnedAgent {
    pub agent: Agent,                    // The agent instance
    pub config: AgentConfig,             // Its configuration
    pub result: Option<String>,          // Its result
}
```

## Usage Guide

### Basic Usage

```rust
use helios_engine::{AutoForest, Config, CalculatorTool, FileReadTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    // Create configuration
    let config = Config::builder()
        .model("gpt-4")
        .api_key("your-api-key")
        .build();

    // Create AutoForest with available tools
    let mut auto_forest = AutoForest::new(config)
        .with_tools(vec![
            Box::new(CalculatorTool),
            Box::new(FileReadTool),
        ])
        .build()
        .await?;

    // Execute a complex task
    let task = "Analyze quarterly sales data and predict next quarter trends";
    let result = auto_forest.execute_task(task).await?;
    
    println!("Result:\n{}", result);
    Ok(())
}
```

### Accessing the Orchestration Plan

After executing a task, you can inspect how AutoForest organized the work:

```rust
let result = auto_forest.execute_task("Complex task").await?;

// Get the plan that was generated
if let Some(plan) = auto_forest.orchestration_plan() {
    println!("Number of agents spawned: {}", plan.num_agents);
    println!("Reasoning: {}", plan.reasoning);
    
    for agent_config in &plan.agents {
        println!("Agent: {} ({})", agent_config.name, agent_config.role);
    }
}
```

### Accessing Spawned Agents

You can also examine the agents that were created:

```rust
for spawned in auto_forest.spawned_agents() {
    println!("Agent: {}", spawned.config.name);
    println!("Role: {}", spawned.config.role);
    println!("Result: {:?}", spawned.result);
}
```

## Use Cases

### 1. Data Analysis
**Task**: "Analyze customer feedback data and identify pain points"

AutoForest might spawn:
- **Sentiment Analyzer**: Processes emotional tone
- **Pattern Recognizer**: Identifies recurring themes
- **Priority Ranker**: Orders issues by importance

### 2. Report Generation
**Task**: "Create a comprehensive market analysis report"

AutoForest might spawn:
- **Market Researcher**: Gathers market data
- **Analyst**: Interprets findings
- **Writer**: Synthesizes into coherent report

### 3. Problem Solving
**Task**: "Troubleshoot performance issues in our application"

AutoForest might spawn:
- **Log Analyzer**: Examines error logs
- **Performance Profiler**: Identifies bottlenecks
- **Solution Designer**: Proposes fixes

### 4. Content Generation
**Task**: "Create marketing copy for different audience segments"

AutoForest might spawn:
- **Copywriter-B2B**: Creates business-focused content
- **Copywriter-Consumer**: Creates consumer-focused content
- **Editor**: Reviews and harmonizes

## Advanced Features

### Custom Tool Assignment

The orchestrator can intelligently assign tools:

```rust
let auto_forest = AutoForest::new(config)
    .with_tools(vec![
        Box::new(CalculatorTool),        // Tool 0
        Box::new(FileReadTool),          // Tool 1
        Box::new(HttpRequestTool),       // Tool 2
        Box::new(TextProcessorTool),     // Tool 3
    ])
    .build()
    .await?;

// The orchestrator will decide which tools go to which agents
```

The `AgentConfig.tool_indices` field specifies which tools each agent receives.

### Result Aggregation

For multi-agent tasks, AutoForest automatically:

1. Collects individual agent results
2. Uses the orchestrator to synthesize findings
3. Produces a unified final answer

```
## Task Execution Summary

**Task**: Analyze quarterly data

### Agent Results:

**Analyst**: Found 15% growth in Q3...
**Forecaster**: Predicting 12% growth in Q4...
**Reporter**: Key findings show...

### Synthesized Analysis:

Combining the analyses, we can conclude...
```

## Configuration

### Creating an AutoForest Instance

```rust
// Step 1: Create or load config
let config = Config::builder()
    .model("gpt-4")
    .temperature(0.7)
    .max_tokens(2048)
    .build();

// Step 2: Create AutoForest with tools
let auto_forest = AutoForest::new(config)
    .with_tools(vec![
        Box::new(CalculatorTool),
        Box::new(FileReadTool),
        // ... more tools
    ])
    .build()
    .await?;

// Step 3: Execute tasks
let result = auto_forest.execute_task("Your task here").await?;
```

## How the Orchestrator Makes Decisions

The orchestrator uses the following factors to decide:

1. **Task Complexity**: More complex tasks get more agents
2. **Tool Count**: More tools allow for specialization
3. **Subtask Identification**: Breaking down into discrete subtasks
4. **Role Specialization**: Assigning experts to specific domains

The orchestrator generates a JSON-based plan that includes:
- Agent count (1-5)
- Individual agent configurations
- Task breakdown
- Reasoning for decisions

## Best Practices

### 1. Provide Clear Task Descriptions

❌ **Poor**: "Analyze this"
✅ **Good**: "Analyze Q3 sales data to identify product categories with declining revenue and forecast Q4 trends"

### 2. Include Relevant Tools

Provide tools that are actually useful for the task:

```rust
// ✅ Good - tools align with task
.with_tools(vec![
    Box::new(CalculatorTool),
    Box::new(FileReadTool),
])

// ❌ Not ideal - irrelevant tools
.with_tools(vec![
    Box::new(HttpRequestTool),  // Not relevant for local analysis
])
```

### 3. Monitor Orchestration Plans

Check the generated plans to understand how AutoForest is organizing work:

```rust
if let Some(plan) = auto_forest.orchestration_plan() {
    println!("Orchestration reasoning: {}", plan.reasoning);
}
```

### 4. Start Simple, Scale Up

Begin with simpler tasks to understand how AutoForest works:

```rust
// Test with simpler task first
let simple_result = auto_forest.execute_task("Calculate 15 * 7 + 20").await?;

// Then try more complex tasks
let complex_result = auto_forest.execute_task("Complex analysis task").await?;
```

## Limitations & Considerations

### Current Limitations

1. **Tool Cloning**: Tools cannot be cloned and assigned individually (all agents work without direct tool access, relying on LLM capabilities)
2. **Sequential Planning**: The orchestrator generates the full plan before execution
3. **Agent Count Cap**: Maximum 5 agents (to avoid excessive parallelization)
4. **No Inter-Agent Communication**: Agents work independently without direct messaging

### Future Enhancements

- Dynamic tool cloning for tool assignment
- Real-time plan adjustment based on agent progress
- Inter-agent communication and collaboration
- Agent team formation based on historical performance
- Hierarchical orchestration (orchestrators managing sub-forests)

## Troubleshooting

### Issue: AutoForest doesn't spawn expected number of agents

**Solution**: Check your task description. Clear, detailed tasks result in better planning.

### Issue: Agents don't have access to needed functionality

**Solution**: Ensure required tools are included in the `with_tools()` call. Agents rely on LLM capabilities if tools aren't available.

### Issue: Results seem incomplete or fragmented

**Solution**: AutoForest should synthesize results automatically. Check the aggregated result section of the output.

## API Reference

### AutoForest Methods

```rust
impl AutoForest {
    // Create a builder
    pub fn new(config: Config) -> AutoForestBuilder

    // Get orchestration plan
    pub fn orchestration_plan(&self) -> Option<&OrchestrationPlan>

    // Get spawned agents
    pub fn spawned_agents(&self) -> &[SpawnedAgent]

    // Execute a task
    pub async fn execute_task(&mut self, task: &str) -> Result<String>
}
```

### AutoForestBuilder Methods

```rust
impl AutoForestBuilder {
    // Set tools
    pub fn with_tools(self, tools: Vec<Box<dyn Tool>>) -> Self

    // Build the orchestrator
    pub async fn build(self) -> Result<AutoForest>
}
```

## Examples

See `examples/auto_forest_demo.rs` for a complete working example.

## FAQ

**Q: How does AutoForest decide on the number of agents?**
A: The orchestrator analyzes task complexity, available tools, and identified subtasks to determine optimal agent count (1-5).

**Q: Can I force a specific number of agents?**
A: Currently, no. AutoForest makes intelligent decisions. You can inspect the plan and provide clearer task descriptions to influence the decision.

**Q: What if I only want one agent?**
A: If the orchestrator determines one agent is sufficient, it will spawn just one. This is common for simpler tasks.

**Q: Can agents communicate with each other?**
A: Currently, agents work independently. Results are aggregated by the orchestrator, but there's no real-time inter-agent messaging.

**Q: How does result aggregation work?**
A: For multiple agents, results are collected and the orchestrator synthesizes them into a cohesive final response.

## Conclusion

AutoForest brings intelligent orchestration to multi-agent systems, automatically handling the complexity of agent spawning, configuration, and coordination. It's ideal for complex tasks that benefit from specialized agents working in parallel.

For more information, see the examples and API documentation.
