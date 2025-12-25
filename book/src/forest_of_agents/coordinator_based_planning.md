# Coordinator-Based Planning

The coordinator-based planning system enables automatic task decomposition and delegation. This is a powerful feature that allows you to create sophisticated multi-agent workflows with minimal effort.

## How It Works

1. **Task Analysis**: The coordinator analyzes the incoming task.
2. **Plan Creation**: The coordinator creates a structured plan with subtasks.
3. **Agent Selection**: The coordinator assigns subtasks to the most appropriate worker agents.
4. **Execution**: The worker agents execute their assigned subtasks.
5. **Result Aggregation**: The coordinator combines the results from the worker agents into a final output.

## Enabling Coordinator Planning

To enable coordinator-based planning, you must do two things:

1. Call the `enable_coordinator_planning()` method on the `ForestBuilder`.
2. Designate one of your agents as the coordinator using the `coordinator_agent()` method.

Here's an example:

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

## Plan Structure

The coordinator creates plans in a JSON format that looks like this:

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

## Custom Coordinator Prompts

You can customize how the coordinator creates plans by providing a custom system prompt. This allows you to tailor the planning process to your specific needs.

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
