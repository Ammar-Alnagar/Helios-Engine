# Forest of Agents - Enhanced Coordinator-Based Planning System

## Overview

The enhanced Forest of Agents feature introduces a sophisticated coordinator-based planning system where a coordinator agent creates detailed task plans, delegates work to specialized agents, and manages a shared task memory that all agents can read from and write to.

## Key Concepts

### 1. Coordinator Agent
The coordinator is responsible for:
- Analyzing complex tasks and breaking them into manageable subtasks
- Creating structured plans with task dependencies
- Assigning tasks to the most appropriate specialized agents
- Synthesizing final results from all completed tasks

### 2. Shared Task Memory
A centralized memory space where:
- The task plan is stored and tracked
- Each agent can see what other agents have completed
- Agents update their results for others to use
- Progress is tracked in real-time

### 3. Task Dependencies
Tasks can have dependencies on other tasks, ensuring:
- Proper execution order
- Sequential workflows where needed
- Parallel execution of independent tasks
- Data flow between dependent tasks

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Coordinator Agent                         │
│  • Creates TaskPlan using create_plan tool                  │
│  • Assigns tasks to specialized agents                      │
│  • Synthesizes final results                                │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────┐
│                   Shared Task Memory                         │
│  • TaskPlan with all tasks and their status                 │
│  • Shared data accessible to all agents                     │
│  • Task results and outputs                                 │
│  • Progress tracking                                        │
└──┬───────────┬───────────┬───────────┬──────────────────────┘
   │           │           │           │
   ▼           ▼           ▼           ▼
┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
│ Agent 1 │ │ Agent 2 │ │ Agent 3 │ │ Agent N │
│         │ │         │ │         │ │         │
│ Updates │ │ Updates │ │ Updates │ │ Updates │
│ Memory  │ │ Memory  │ │ Memory  │ │ Memory  │
└─────────┘ └─────────┘ └─────────┘ └─────────┘
```

## Core Components

### TaskPlan
Represents the overall collaborative task plan:
```rust
pub struct TaskPlan {
    pub plan_id: String,
    pub objective: String,
    pub tasks: Vec<TaskItem>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

### TaskItem
Represents an individual task within the plan:
```rust
pub struct TaskItem {
    pub id: String,
    pub description: String,
    pub assigned_to: AgentId,
    pub status: TaskStatus,
    pub result: Option<String>,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, String>,
}
```

### TaskStatus
Tracks the state of each task:
```rust
pub enum TaskStatus {
    Pending,      // Not yet started
    InProgress,   // Currently being worked on
    Completed,    // Successfully completed
    Failed,       // Failed to complete
}
```

## New Tools

### 1. create_plan
**Purpose**: Allows the coordinator to create a structured task plan

**Parameters**:
- `objective` (string, required): The overall goal
- `tasks` (string, required): JSON array of task objects

**Task Object Structure**:
```json
{
  "id": "task_1",
  "description": "Gather research on renewable energy",
  "assigned_to": "researcher",
  "dependencies": []
}
```

**Example Usage**:
```rust
// The coordinator agent uses this tool automatically
// when you call execute_collaborative_task
```

### 2. update_task_memory
**Purpose**: Allows agents to save their results to shared memory

**Parameters**:
- `task_id` (string, required): The ID of the task being updated
- `result` (string, required): The output/findings from the task
- `data` (string, optional): Additional data to share with other agents

**Example Usage**:
```rust
// Agents call this tool to update shared memory
update_task_memory(
    task_id: "task_1",
    result: "Research complete: Found 5 key benefits...",
    data: "Key metrics: 30% efficiency increase, 50% cost reduction"
)
```

### 3. Existing Tools Still Available
- `send_message`: Send direct messages between agents
- `delegate_task`: Delegate specific work (now integrated with planning)
- `share_context`: Share general information

## Workflow Phases

### Phase 1: Planning
1. Coordinator receives the overall task
2. Coordinator analyzes and breaks down into subtasks
3. Coordinator uses `create_plan` tool to create structured plan
4. Plan is stored in shared memory

### Phase 2: Execution
1. System identifies tasks with no dependencies (ready to execute)
2. Assigned agents receive their tasks with context:
   - Shared memory contents (previous results)
   - Their specific task description
   - Overall objective
3. Agents complete tasks and use `update_task_memory` to save results
4. System marks task as completed
5. Repeat until all tasks are done

### Phase 3: Synthesis
1. Coordinator receives summary of all completed tasks
2. Coordinator synthesizes comprehensive final result
3. Final result is returned to the user

## Usage Example

### Basic Example

```rust
use helios_engine::{Agent, Config, ForestBuilder};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;

    // Create forest with specialized agents
    let mut forest = ForestBuilder::new()
        .config(config)
        .agent(
            "coordinator".to_string(),
            Agent::builder("coordinator")
                .system_prompt("You are a coordinator who creates plans...")
        )
        .agent(
            "researcher".to_string(),
            Agent::builder("researcher")
                .system_prompt("You are a researcher...")
        )
        .agent(
            "writer".to_string(),
            Agent::builder("writer")
                .system_prompt("You are a writer...")
        )
        .max_iterations(20)
        .build()
        .await?;

    // Execute collaborative task with planning
    let result = forest
        .execute_collaborative_task(
            &"coordinator".to_string(),
            "Create a comprehensive guide on renewable energy".to_string(),
            vec!["researcher".to_string(), "writer".to_string()],
        )
        .await?;

    println!("Result: {}", result);
    Ok(())
}
```

### Advanced Example with Dependencies

The coordinator can create plans with dependencies:

```json
[
  {
    "id": "task_1",
    "description": "Research renewable energy sources",
    "assigned_to": "researcher",
    "dependencies": []
  },
  {
    "id": "task_2",
    "description": "Analyze research data",
    "assigned_to": "analyst",
    "dependencies": ["task_1"]
  },
  {
    "id": "task_3",
    "description": "Write content based on analysis",
    "assigned_to": "writer",
    "dependencies": ["task_2"]
  },
  {
    "id": "task_4",
    "description": "Review final content",
    "assigned_to": "reviewer",
    "dependencies": ["task_3"]
  }
]
```

This creates a sequential workflow: Research → Analysis → Writing → Review

## Best Practices

### 1. Coordinator System Prompt
Give the coordinator clear instructions:
```rust
.system_prompt(
    "You are a coordinator. Break complex tasks into:\n\
    1. Research/data gathering tasks\n\
    2. Analysis/processing tasks\n\
    3. Content creation tasks\n\
    4. Review/quality assurance tasks\n\n\
    Use the create_plan tool to structure the work."
)
```

### 2. Agent System Prompts
Instruct agents to use shared memory:
```rust
.system_prompt(
    "You are a researcher. When completing tasks:\n\
    1. Review shared memory for context\n\
    2. Complete your research thoroughly\n\
    3. Use update_task_memory to save findings\n\
    4. Include data that other agents might need"
)
```

### 3. Task Granularity
- Break tasks into focused, manageable pieces
- Each task should have a clear output
- Avoid tasks that are too broad or too narrow
- Aim for 3-7 tasks per plan

### 4. Dependencies
- Use dependencies for sequential workflows
- Keep parallel tasks independent when possible
- Don't create circular dependencies
- Test dependency chains before deployment

### 5. Shared Memory Usage
- Agents should read shared memory for context
- Update memory with relevant results
- Include data that downstream tasks need
- Use descriptive keys for shared data

## Monitoring and Debugging

### View Task Plan Status
```rust
let context = forest.get_shared_context().await;
if let Some(plan) = context.get_plan() {
    let (completed, total) = plan.get_progress();
    println!("Progress: {}/{} tasks completed", completed, total);
    
    for task in &plan.tasks {
        println!("{}: {} - {:?}", task.id, task.description, task.status);
    }
}
```

### Access Shared Data
```rust
let context = forest.get_shared_context().await;
for (key, value) in &context.data {
    println!("{}: {}", key, value);
}
```

### View Task Results
```rust
if let Some(plan) = context.get_plan() {
    for task in &plan.tasks {
        if let Some(result) = &task.result {
            println!("[{}] {}: {}", task.assigned_to, task.description, result);
        }
    }
}
```

## Performance Considerations

### Iteration Limits
- Set appropriate `max_iterations` for complex tasks
- The system uses `max_iterations * 2` for task execution
- Monitor iteration usage in logs

### Memory Usage
- Shared memory grows with task results
- Clear plans between different collaborative tasks
- Use concise result strings when possible

### Parallel Execution
- Independent tasks (no dependencies) can be parallelized
- Current implementation is sequential for reliability
- Future versions may add parallel execution

## Comparison: Old vs New Approach

### Old Approach (Simple Delegation)
```
User Request → Coordinator → Delegates ad-hoc → Agents respond → Done
```
- No structured planning
- Limited visibility into progress
- No dependency management
- Results not systematically shared

### New Approach (Coordinator-Based Planning)
```
User Request → Coordinator creates plan → 
  → Execute tasks in order (respecting dependencies) →
  → Each agent updates shared memory →
  → Coordinator synthesizes final result
```
- Structured task breakdown
- Clear progress tracking
- Dependency management
- Systematic result sharing
- Better quality outcomes

## Real-World Use Cases

### 1. Content Creation Pipeline
```
Research → Outline → Draft → Edit → Review → Publish
```

### 2. Software Development
```
Requirements → Design → Implementation → Testing → Documentation
```

### 3. Data Analysis Project
```
Data Collection → Cleaning → Analysis → Visualization → Report
```

### 4. Business Strategy
```
Market Research → SWOT Analysis → Strategy Formulation → Review
```

### 5. Scientific Research
```
Literature Review → Hypothesis → Experiment → Analysis → Paper
```

## Troubleshooting

### Problem: Plan not created
**Solution**: Check coordinator's system prompt includes instructions to use `create_plan` tool

### Problem: Tasks not executing
**Solution**: Verify all assigned agents exist in the forest

### Problem: Dependencies not working
**Solution**: Ensure dependency task IDs match exactly

### Problem: Shared memory not updating
**Solution**: Check agents are using `update_task_memory` tool correctly

### Problem: Iteration limit reached
**Solution**: Increase `max_iterations` or simplify the task plan

## API Reference

See the main documentation for detailed API information on:
- `ForestOfAgents::execute_collaborative_task()`
- `TaskPlan` and `TaskItem` structures
- `CreatePlanTool`, `UpdateTaskMemoryTool`
- `SharedContext` methods

## Examples

Run the comprehensive example:
```bash
cargo run --example forest_with_coordinator
```

Run the basic forest example:
```bash
cargo run --example forest_of_agents
```

## Future Enhancements

Potential improvements:
- [ ] Parallel task execution for independent tasks
- [ ] Task retry logic for failed tasks
- [ ] Task prioritization
- [ ] Dynamic replanning based on results
- [ ] Visualization of task execution flow
- [ ] Metrics and analytics dashboard
- [ ] Task templates for common workflows
- [ ] Agent workload balancing

## Conclusion

The enhanced coordinator-based planning system transforms the Forest of Agents into a powerful multi-agent collaboration framework suitable for complex, real-world tasks. By combining structured planning, shared memory, and dependency management, it enables sophisticated workflows while maintaining simplicity for users.
