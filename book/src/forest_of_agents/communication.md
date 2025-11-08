# Agent Communication

Agents in a Forest of Agents can communicate with each other to share information, delegate tasks, and collaborate on complex problems.

## `SendMessageTool`

The primary mechanism for agent communication is the `SendMessageTool`. This tool allows an agent to send a message to another agent in the same forest.

### Usage

To use the `SendMessageTool`, you must first register it with an agent.

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

Once the tool is registered, the agent can use it to send messages to other agents. The agent will typically do this automatically when it determines that it needs to communicate with another agent to complete its task.

The `SendMessageTool` takes the following parameters:

- **`to_agent`**: The ID of the agent to send the message to.
- **`message`**: The message to send.

Here's an example of the JSON that an agent might generate to use the `SendMessageTool`:

```json
{
  "to_agent": "agent2",
  "message": "Please analyze this data: [1, 2, 3, 4, 5]"
}
```

## Accessing Agent Results

You can also access the results of an agent's work by getting the agent from the forest and inspecting its chat history.

```rust
// Get a specific agent's last response
if let Some(agent) = forest.get_agent("researcher") {
    let history = agent.chat_session().get_messages();
    // Process history...
}

// List all agents
let agent_ids = forest.list_agents();
for id in agent_ids {
    println!("Agent: {}", id);
}
```
