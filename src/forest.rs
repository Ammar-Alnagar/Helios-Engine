//! # Forest of Agents Module
//!
//! This module implements the "Forest of Agents" feature, which allows multiple agents
//! to interact with each other, share context, and collaborate on tasks.
//!
//! The ForestOfAgents struct manages a collection of agents and provides mechanisms
//! for inter-agent communication and coordination.

use crate::agent::{Agent, AgentBuilder};
use crate::config::Config;
use crate::error::{HeliosError, Result};
use crate::tools::{Tool, ToolParameter, ToolResult};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A unique identifier for an agent in the forest.
pub type AgentId = String;

/// A message sent between agents in the forest.
#[derive(Debug, Clone)]
pub struct ForestMessage {
    /// The ID of the sender agent.
    pub from: AgentId,
    /// The ID of the recipient agent (None for broadcast).
    pub to: Option<AgentId>,
    /// The message content.
    pub content: String,
    /// Optional metadata associated with the message.
    pub metadata: HashMap<String, String>,
    /// Timestamp of the message.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ForestMessage {
    /// Creates a new forest message.
    pub fn new(from: AgentId, to: Option<AgentId>, content: String) -> Self {
        Self {
            from,
            to,
            content,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Creates a broadcast message to all agents.
    pub fn broadcast(from: AgentId, content: String) -> Self {
        Self::new(from, None, content)
    }

    /// Adds metadata to the message.
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Shared context that can be accessed by all agents in the forest.
#[derive(Debug, Clone)]
pub struct SharedContext {
    /// Key-value store for shared data.
    pub data: HashMap<String, Value>,
    /// Message history between agents.
    pub message_history: Vec<ForestMessage>,
    /// Global metadata.
    pub metadata: HashMap<String, String>,
}

impl SharedContext {
    /// Creates a new empty shared context.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            message_history: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Sets a value in the shared context.
    pub fn set(&mut self, key: String, value: Value) {
        self.data.insert(key, value);
    }

    /// Gets a value from the shared context.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// Removes a value from the shared context.
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.data.remove(key)
    }

    /// Adds a message to the history.
    pub fn add_message(&mut self, message: ForestMessage) {
        self.message_history.push(message);
    }

    /// Gets recent messages (last N messages).
    pub fn get_recent_messages(&self, limit: usize) -> &[ForestMessage] {
        let len = self.message_history.len();
        let start = if len > limit { len - limit } else { 0 };
        &self.message_history[start..]
    }
}

impl Default for SharedContext {
    fn default() -> Self {
        Self::new()
    }
}

/// The main Forest of Agents structure that manages multiple agents.
pub struct ForestOfAgents {
    /// The agents in the forest, keyed by their IDs.
    agents: HashMap<AgentId, Agent>,
    /// Shared context accessible to all agents.
    shared_context: Arc<RwLock<SharedContext>>,
    /// Message queue for inter-agent communication.
    message_queue: Arc<RwLock<Vec<ForestMessage>>>,
    /// Maximum number of iterations for agent interactions.
    max_iterations: usize,
}

impl ForestOfAgents {
    /// Creates a new empty Forest of Agents.
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            shared_context: Arc::new(RwLock::new(SharedContext::new())),
            message_queue: Arc::new(RwLock::new(Vec::new())),
            max_iterations: 10,
        }
    }

    /// Creates a new Forest of Agents with the specified max iterations.
    pub fn with_max_iterations(max_iterations: usize) -> Self {
        Self {
            max_iterations,
            ..Self::new()
        }
    }

    /// Adds an agent to the forest.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the agent
    /// * `agent` - The agent to add
    ///
    /// # Returns
    ///
    /// Returns an error if an agent with the same ID already exists.
    pub fn add_agent(&mut self, id: AgentId, mut agent: Agent) -> Result<()> {
        if self.agents.contains_key(&id) {
            return Err(HeliosError::AgentError(format!(
                "Agent with ID '{}' already exists",
                id
            )));
        }

        // Register communication tools for this agent
        let send_message_tool = Box::new(SendMessageTool::new(
            id.clone(),
            Arc::clone(&self.message_queue),
            Arc::clone(&self.shared_context),
        ));
        agent.register_tool(send_message_tool);

        let delegate_task_tool = Box::new(DelegateTaskTool::new(
            id.clone(),
            Arc::clone(&self.message_queue),
            Arc::clone(&self.shared_context),
        ));
        agent.register_tool(delegate_task_tool);

        let share_context_tool = Box::new(ShareContextTool::new(
            id.clone(),
            Arc::clone(&self.shared_context),
        ));
        agent.register_tool(share_context_tool);

        self.agents.insert(id, agent);
        Ok(())
    }

    /// Removes an agent from the forest.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the agent to remove
    ///
    /// # Returns
    ///
    /// Returns the removed agent if it existed.
    pub fn remove_agent(&mut self, id: &AgentId) -> Option<Agent> {
        self.agents.remove(id)
    }

    /// Gets a reference to an agent by ID.
    pub fn get_agent(&self, id: &AgentId) -> Option<&Agent> {
        self.agents.get(id)
    }

    /// Gets a mutable reference to an agent by ID.
    pub fn get_agent_mut(&mut self, id: &AgentId) -> Option<&mut Agent> {
        self.agents.get_mut(id)
    }

    /// Lists all agent IDs in the forest.
    pub fn list_agents(&self) -> Vec<AgentId> {
        self.agents.keys().cloned().collect()
    }

    /// Sends a message from one agent to another.
    ///
    /// # Arguments
    ///
    /// * `from` - ID of the sending agent
    /// * `to` - ID of the receiving agent (None for broadcast)
    /// * `content` - Message content
    ///
    /// # Returns
    ///
    /// Returns an error if the sender doesn't exist.
    pub async fn send_message(
        &self,
        from: &AgentId,
        to: Option<&AgentId>,
        content: String,
    ) -> Result<()> {
        if !self.agents.contains_key(from) {
            return Err(HeliosError::AgentError(format!(
                "Agent '{}' not found",
                from
            )));
        }

        let message = if let Some(to_id) = to {
            ForestMessage::new(from.clone(), Some(to_id.clone()), content)
        } else {
            ForestMessage::broadcast(from.clone(), content)
        };

        let mut queue = self.message_queue.write().await;
        queue.push(message.clone());

        // Also add to shared context history
        let mut context = self.shared_context.write().await;
        context.add_message(message);

        Ok(())
    }

    /// Processes pending messages in the queue.
    pub async fn process_messages(&mut self) -> Result<()> {
        let messages: Vec<ForestMessage> = {
            let mut queue = self.message_queue.write().await;
            queue.drain(..).collect()
        };

        for message in messages {
            if let Some(to_id) = &message.to {
                // Direct message
                if let Some(agent) = self.agents.get_mut(to_id) {
                    // Add the message as a user message to the agent's chat session
                    let formatted_message =
                        format!("Message from {}: {}", message.from, message.content);
                    agent.chat_session_mut().add_user_message(formatted_message);
                }
            } else {
                // Broadcast message - send to all agents except sender
                for (agent_id, agent) in &mut self.agents {
                    if agent_id != &message.from {
                        let formatted_message =
                            format!("Broadcast from {}: {}", message.from, message.content);
                        agent.chat_session_mut().add_user_message(formatted_message);
                    }
                }
            }
        }

        Ok(())
    }

    /// Executes a collaborative task across multiple agents.
    ///
    /// # Arguments
    ///
    /// * `initiator` - ID of the agent initiating the task
    /// * `task_description` - Description of the task
    /// * `involved_agents` - IDs of agents to involve in the task
    ///
    /// # Returns
    ///
    /// Returns the final result from the collaborative process.
    pub async fn execute_collaborative_task(
        &mut self,
        initiator: &AgentId,
        task_description: String,
        involved_agents: Vec<AgentId>,
    ) -> Result<String> {
        // Verify all involved agents exist
        for agent_id in &involved_agents {
            if !self.agents.contains_key(agent_id) {
                return Err(HeliosError::AgentError(format!(
                    "Agent '{}' not found",
                    agent_id
                )));
            }
        }

        if !self.agents.contains_key(initiator) {
            return Err(HeliosError::AgentError(format!(
                "Initiator agent '{}' not found",
                initiator
            )));
        }

        // Set up the collaborative context
        {
            let mut context = self.shared_context.write().await;
            context.set(
                "current_task".to_string(),
                Value::String(task_description.clone()),
            );
            context.set(
                "involved_agents".to_string(),
                Value::Array(
                    involved_agents
                        .iter()
                        .map(|id| Value::String(id.clone()))
                        .collect(),
                ),
            );
            context.set(
                "task_status".to_string(),
                Value::String("in_progress".to_string()),
            );
        }

        // Start the collaboration by having the initiator break down the task
        let initiator_agent = self.agents.get_mut(initiator).unwrap();
        let breakdown_prompt = format!(
            "You are working with other agents to complete this task: {}\n\
            The other agents involved are: {}\n\
            Please break down this task into subtasks that can be delegated to the other agents, \
            or coordinate with them to work together. Use the available communication tools to \
            delegate tasks, share information, and collaborate.",
            task_description,
            involved_agents.join(", ")
        );

        let mut result = initiator_agent.chat(breakdown_prompt).await?;

        // Process agent interactions for up to max_iterations
        for _iteration in 0..self.max_iterations {
            // Process any pending messages
            self.process_messages().await?;

            // Check if any agents want to respond or continue the collaboration
            let mut active_responses = Vec::new();

            for agent_id in &involved_agents {
                if let Some(agent) = self.agents.get_mut(agent_id) {
                    // Check if agent has new messages to process
                    if !agent.chat_session().messages.is_empty() {
                        let last_message = &agent.chat_session().messages.last().unwrap();
                        if last_message.role == crate::chat::Role::User {
                            // Agent has a new message to respond to
                            let response = agent
                                .chat("Continue collaborating on the current task.")
                                .await?;
                            active_responses.push((agent_id.clone(), response));
                        }
                    }
                }
            }

            if active_responses.is_empty() {
                break; // No more active responses
            }

            // Process responses and continue collaboration
            for (agent_id, response) in active_responses {
                result = format!("{}\n\nAgent {}: {}", result, agent_id, response);
            }
        }

        // Mark task as completed
        {
            let mut context = self.shared_context.write().await;
            context.set(
                "task_status".to_string(),
                Value::String("completed".to_string()),
            );
        }

        Ok(result)
    }

    /// Gets the shared context.
    pub async fn get_shared_context(&self) -> SharedContext {
        self.shared_context.read().await.clone()
    }

    /// Sets a value in the shared context.
    pub async fn set_shared_context(&self, key: String, value: Value) {
        let mut context = self.shared_context.write().await;
        context.set(key, value);
    }
}

impl Default for ForestOfAgents {
    fn default() -> Self {
        Self::new()
    }
}

/// A tool that allows agents to send messages to other agents.
pub struct SendMessageTool {
    agent_id: AgentId,
    message_queue: Arc<RwLock<Vec<ForestMessage>>>,
    shared_context: Arc<RwLock<SharedContext>>,
}

impl SendMessageTool {
    /// Creates a new SendMessageTool.
    pub fn new(
        agent_id: AgentId,
        message_queue: Arc<RwLock<Vec<ForestMessage>>>,
        shared_context: Arc<RwLock<SharedContext>>,
    ) -> Self {
        Self {
            agent_id,
            message_queue,
            shared_context,
        }
    }
}

#[async_trait::async_trait]
impl Tool for SendMessageTool {
    fn name(&self) -> &str {
        "send_message"
    }

    fn description(&self) -> &str {
        "Send a message to another agent or broadcast to all agents in the forest."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "to".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "ID of the recipient agent (leave empty for broadcast)".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "message".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "The message content to send".to_string(),
                required: Some(true),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let message = args
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'message' parameter".to_string()))?
            .to_string();

        let to = args
            .get("to")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let forest_message = if let Some(to_id) = &to {
            ForestMessage::new(self.agent_id.clone(), Some(to_id.clone()), message)
        } else {
            ForestMessage::broadcast(self.agent_id.clone(), message)
        };

        {
            let mut queue = self.message_queue.write().await;
            queue.push(forest_message.clone());
        }

        {
            let mut context = self.shared_context.write().await;
            context.add_message(forest_message);
        }

        Ok(ToolResult::success("Message sent successfully"))
    }
}

/// A tool that allows agents to delegate tasks to other agents.
pub struct DelegateTaskTool {
    agent_id: AgentId,
    message_queue: Arc<RwLock<Vec<ForestMessage>>>,
    shared_context: Arc<RwLock<SharedContext>>,
}

impl DelegateTaskTool {
    /// Creates a new DelegateTaskTool.
    pub fn new(
        agent_id: AgentId,
        message_queue: Arc<RwLock<Vec<ForestMessage>>>,
        shared_context: Arc<RwLock<SharedContext>>,
    ) -> Self {
        Self {
            agent_id,
            message_queue,
            shared_context,
        }
    }
}

#[async_trait::async_trait]
impl Tool for DelegateTaskTool {
    fn name(&self) -> &str {
        "delegate_task"
    }

    fn description(&self) -> &str {
        "Delegate a specific task to another agent for execution."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "to".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "ID of the agent to delegate the task to".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "task".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Description of the task to delegate".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "context".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Additional context or requirements for the task".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let to = args
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'to' parameter".to_string()))?;

        let task = args
            .get("task")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'task' parameter".to_string()))?;

        let context = args.get("context").and_then(|v| v.as_str()).unwrap_or("");

        let message = if context.is_empty() {
            format!("Task delegated: {}", task)
        } else {
            format!("Task delegated: {}\nContext: {}", task, context)
        };

        let forest_message =
            ForestMessage::new(self.agent_id.clone(), Some(to.to_string()), message)
                .with_metadata("type".to_string(), "task_delegation".to_string())
                .with_metadata("task".to_string(), task.to_string());

        {
            let mut queue = self.message_queue.write().await;
            queue.push(forest_message.clone());
        }

        {
            let mut context_lock = self.shared_context.write().await;
            context_lock.add_message(forest_message);
        }

        Ok(ToolResult::success(format!(
            "Task delegated to agent '{}'",
            to
        )))
    }
}

/// A tool that allows agents to share information in the shared context.
pub struct ShareContextTool {
    agent_id: AgentId,
    shared_context: Arc<RwLock<SharedContext>>,
}

impl ShareContextTool {
    /// Creates a new ShareContextTool.
    pub fn new(agent_id: AgentId, shared_context: Arc<RwLock<SharedContext>>) -> Self {
        Self {
            agent_id,
            shared_context,
        }
    }
}

#[async_trait::async_trait]
impl Tool for ShareContextTool {
    fn name(&self) -> &str {
        "share_context"
    }

    fn description(&self) -> &str {
        "Share information in the shared context that all agents can access."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "key".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Key for the shared information".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "value".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Value to share".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "description".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Description of what this information represents".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let key = args
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'key' parameter".to_string()))?;

        let value = args
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'value' parameter".to_string()))?;

        let description = args
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let mut context = self.shared_context.write().await;

        // Store the value
        context.set(key.to_string(), Value::String(value.to_string()));

        // Add metadata about who shared it and when
        let metadata_key = format!("{}_metadata", key);
        let metadata = serde_json::json!({
            "shared_by": self.agent_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "description": description
        });
        context.set(metadata_key, metadata);

        Ok(ToolResult::success(format!(
            "Information shared with key '{}'",
            key
        )))
    }
}

/// Builder for creating a Forest of Agents with multiple agents.
pub struct ForestBuilder {
    config: Option<Config>,
    agents: Vec<(AgentId, AgentBuilder)>,
    max_iterations: usize,
}

impl ForestBuilder {
    /// Creates a new ForestBuilder.
    pub fn new() -> Self {
        Self {
            config: None,
            agents: Vec::new(),
            max_iterations: 10,
        }
    }

    /// Sets the configuration for all agents in the forest.
    pub fn config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    /// Adds an agent to the forest with a builder.
    pub fn agent(mut self, id: AgentId, builder: AgentBuilder) -> Self {
        self.agents.push((id, builder));
        self
    }

    /// Sets the maximum iterations for agent interactions.
    pub fn max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Builds the Forest of Agents.
    pub async fn build(self) -> Result<ForestOfAgents> {
        let config = self
            .config
            .ok_or_else(|| HeliosError::AgentError("Config is required".to_string()))?;

        let mut forest = ForestOfAgents::with_max_iterations(self.max_iterations);

        for (id, builder) in self.agents {
            let agent = builder.config(config.clone()).build().await?;
            forest.add_agent(id, agent)?;
        }

        Ok(forest)
    }
}

impl Default for ForestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::tools::Tool;
    use serde_json::Value;

    /// Tests basic ForestOfAgents creation and agent management.
    #[tokio::test]
    async fn test_forest_creation_and_agent_management() {
        let mut forest = ForestOfAgents::new();
        let config = Config::new_default();

        // Create and add agents
        let agent1 = Agent::builder("agent1")
            .config(config.clone())
            .system_prompt("You are agent 1")
            .build()
            .await
            .unwrap();

        let agent2 = Agent::builder("agent2")
            .config(config)
            .system_prompt("You are agent 2")
            .build()
            .await
            .unwrap();

        // Add agents to forest
        forest.add_agent("agent1".to_string(), agent1).unwrap();
        forest.add_agent("agent2".to_string(), agent2).unwrap();

        // Test agent listing
        let agents = forest.list_agents();
        assert_eq!(agents.len(), 2);
        assert!(agents.contains(&"agent1".to_string()));
        assert!(agents.contains(&"agent2".to_string()));

        // Test agent retrieval
        assert!(forest.get_agent(&"agent1".to_string()).is_some());
        assert!(forest.get_agent(&"agent3".to_string()).is_none());

        // Test duplicate agent addition
        let agent3 = Agent::builder("agent3")
            .config(Config::new_default())
            .build()
            .await
            .unwrap();
        let result = forest.add_agent("agent1".to_string(), agent3);
        assert!(result.is_err());

        // Test agent removal
        let removed = forest.remove_agent(&"agent1".to_string());
        assert!(removed.is_some());
        assert_eq!(forest.list_agents().len(), 1);
        assert!(forest.get_agent(&"agent1".to_string()).is_none());
    }

    /// Tests message sending between agents.
    #[tokio::test]
    async fn test_message_sending() {
        let mut forest = ForestOfAgents::new();
        let config = Config::new_default();

        // Create and add agents
        let agent1 = Agent::builder("alice")
            .config(config.clone())
            .build()
            .await
            .unwrap();

        let agent2 = Agent::builder("bob").config(config).build().await.unwrap();

        forest.add_agent("alice".to_string(), agent1).unwrap();
        forest.add_agent("bob".to_string(), agent2).unwrap();

        // Test direct message
        forest
            .send_message(
                &"alice".to_string(),
                Some(&"bob".to_string()),
                "Hello Bob!".to_string(),
            )
            .await
            .unwrap();

        // Process messages
        forest.process_messages().await.unwrap();

        // Check that Bob received the message
        let bob = forest.get_agent(&"bob".to_string()).unwrap();
        let messages = bob.chat_session().messages.clone();
        assert!(messages.len() >= 1);
        let last_message = messages.last().unwrap();
        assert_eq!(last_message.role, crate::chat::Role::User);
        assert!(last_message
            .content
            .contains("Message from alice: Hello Bob!"));

        // Test broadcast message
        let alice_message_count_before = forest.get_agent(&"alice".to_string()).unwrap().chat_session().messages.len();
        forest
            .send_message(&"alice".to_string(), None, "Hello everyone!".to_string())
            .await
            .unwrap();
        forest.process_messages().await.unwrap();

        // Check that Bob received the broadcast, but Alice did not
        let alice = forest.get_agent(&"alice".to_string()).unwrap();
        assert_eq!(alice.chat_session().messages.len(), alice_message_count_before);

        let bob = forest.get_agent(&"bob".to_string()).unwrap();
        let bob_messages = bob.chat_session().messages.clone();
        let bob_last = bob_messages.last().unwrap();
        assert!(bob_last
            .content
            .contains("Broadcast from alice: Hello everyone!"));
    }

    /// Tests the SendMessageTool functionality.
    #[tokio::test]
    async fn test_send_message_tool() {
        let message_queue = Arc::new(RwLock::new(Vec::new()));
        let shared_context = Arc::new(RwLock::new(SharedContext::new()));

        let tool = SendMessageTool::new(
            "alice".to_string(),
            Arc::clone(&message_queue),
            Arc::clone(&shared_context),
        );

        // Test sending a direct message
        let args = serde_json::json!({
            "to": "bob",
            "message": "Test message"
        });

        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "Message sent successfully");

        // Check message queue
        let queue = message_queue.read().await;
        assert_eq!(queue.len(), 1);
        let message = &queue[0];
        assert_eq!(message.from, "alice");
        assert_eq!(message.to, Some("bob".to_string()));
        assert_eq!(message.content, "Test message");

        // Check shared context
        let context = shared_context.read().await;
        let messages = context.get_recent_messages(10);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].from, "alice");

        // Test broadcast message
        let args = serde_json::json!({
            "message": "Broadcast test"
        });

        tool.execute(args).await.unwrap();

        let queue = message_queue.read().await;
        assert_eq!(queue.len(), 2);
        let broadcast_message = &queue[1];
        assert_eq!(broadcast_message.from, "alice");
        assert!(broadcast_message.to.is_none());
        assert_eq!(broadcast_message.content, "Broadcast test");
    }

    /// Tests the DelegateTaskTool functionality.
    #[tokio::test]
    async fn test_delegate_task_tool() {
        let message_queue = Arc::new(RwLock::new(Vec::new()));
        let shared_context = Arc::new(RwLock::new(SharedContext::new()));

        let tool = DelegateTaskTool::new(
            "manager".to_string(),
            Arc::clone(&message_queue),
            Arc::clone(&shared_context),
        );

        // Test task delegation
        let args = serde_json::json!({
            "to": "worker",
            "task": "Analyze the data",
            "context": "Use statistical methods"
        });

        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "Task delegated to agent 'worker'");

        // Check message queue
        let queue = message_queue.read().await;
        assert_eq!(queue.len(), 1);
        let message = &queue[0];
        assert_eq!(message.from, "manager");
        assert_eq!(message.to, Some("worker".to_string()));
        assert!(message.content.contains("Task delegated: Analyze the data"));
        assert!(message.content.contains("Context: Use statistical methods"));

        // Check metadata
        assert_eq!(
            message.metadata.get("type"),
            Some(&"task_delegation".to_string())
        );
        assert_eq!(
            message.metadata.get("task"),
            Some(&"Analyze the data".to_string())
        );
    }

    /// Tests the ShareContextTool functionality.
    #[tokio::test]
    async fn test_share_context_tool() {
        let shared_context = Arc::new(RwLock::new(SharedContext::new()));

        let tool = ShareContextTool::new("researcher".to_string(), Arc::clone(&shared_context));

        // Test sharing context
        let args = serde_json::json!({
            "key": "findings",
            "value": "Temperature affects reaction rate",
            "description": "Key experimental finding"
        });

        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "Information shared with key 'findings'");

        // Check shared context
        let context = shared_context.read().await;
        assert_eq!(
            context.get("findings"),
            Some(&Value::String(
                "Temperature affects reaction rate".to_string()
            ))
        );

        // Check metadata
        let metadata = context.get("findings_metadata").unwrap();
        let metadata_obj = metadata.as_object().unwrap();
        assert_eq!(metadata_obj.get("shared_by").unwrap(), "researcher");
        assert_eq!(
            metadata_obj.get("description").unwrap(),
            "Key experimental finding"
        );
        assert!(metadata_obj.contains_key("timestamp"));
    }

    /// Tests the SharedContext functionality.
    #[tokio::test]
    async fn test_shared_context() {
        let mut context = SharedContext::new();

        // Test setting and getting values
        context.set("key1".to_string(), Value::String("value1".to_string()));
        context.set("key2".to_string(), Value::Number(42.into()));

        assert_eq!(
            context.get("key1"),
            Some(&Value::String("value1".to_string()))
        );
        assert_eq!(context.get("key2"), Some(&Value::Number(42.into())));
        assert_eq!(context.get("key3"), None);

        // Test message history
        let msg1 = ForestMessage::new(
            "alice".to_string(),
            Some("bob".to_string()),
            "Hello".to_string(),
        );
        let msg2 = ForestMessage::broadcast("bob".to_string(), "Hi everyone".to_string());

        context.add_message(msg1);
        context.add_message(msg2);

        let messages = context.get_recent_messages(10);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].from, "alice");
        assert_eq!(messages[1].from, "bob");

        // Test removing values
        let removed = context.remove("key1");
        assert_eq!(removed, Some(Value::String("value1".to_string())));
        assert_eq!(context.get("key1"), None);
    }

    /// Tests collaborative task execution.
    #[tokio::test]
    async fn test_collaborative_task() {
        let mut forest = ForestOfAgents::new();
        let config = Config::new_default();

        // Create agents with different roles
        let coordinator = Agent::builder("coordinator")
            .config(config.clone())
            .system_prompt(
                "You are a task coordinator. Break down tasks and delegate to specialists.",
            )
            .build()
            .await
            .unwrap();

        let researcher = Agent::builder("researcher")
            .config(config.clone())
            .system_prompt("You are a researcher. Gather and analyze information.")
            .build()
            .await
            .unwrap();

        let writer = Agent::builder("writer")
            .config(config)
            .system_prompt("You are a writer. Create clear, well-structured content.")
            .build()
            .await
            .unwrap();

        forest
            .add_agent("coordinator".to_string(), coordinator)
            .unwrap();
        forest
            .add_agent("researcher".to_string(), researcher)
            .unwrap();
        forest.add_agent("writer".to_string(), writer).unwrap();

        // Execute collaborative task
        let result = forest
            .execute_collaborative_task(
                &"coordinator".to_string(),
                "Create a report on climate change impacts".to_string(),
                vec!["researcher".to_string(), "writer".to_string()],
            )
            .await;

        // The task should complete without error
        assert!(result.is_ok());
        let result_content = result.unwrap();
        assert!(!result_content.is_empty());

        // Check shared context was updated
        let context = forest.get_shared_context().await;
        assert_eq!(
            context.get("task_status"),
            Some(&Value::String("completed".to_string()))
        );
        assert!(context.get("current_task").is_some());
        assert!(context.get("involved_agents").is_some());
    }

    /// Tests the ForestBuilder functionality.
    #[tokio::test]
    async fn test_forest_builder() {
        let config = Config::new_default();

        let forest = ForestBuilder::new()
            .config(config)
            .agent(
                "agent1".to_string(),
                Agent::builder("agent1").system_prompt("Agent 1 prompt"),
            )
            .agent(
                "agent2".to_string(),
                Agent::builder("agent2").system_prompt("Agent 2 prompt"),
            )
            .max_iterations(5)
            .build()
            .await
            .unwrap();

        assert_eq!(forest.list_agents().len(), 2);
        assert!(forest.get_agent(&"agent1".to_string()).is_some());
        assert!(forest.get_agent(&"agent2".to_string()).is_some());
        assert_eq!(forest.max_iterations, 5);
    }

    /// Tests error handling in ForestOfAgents.
    #[tokio::test]
    async fn test_forest_error_handling() {
        let mut forest = ForestOfAgents::new();

        // Test sending message from non-existent agent
        let result = forest
            .send_message(
                &"nonexistent".to_string(),
                Some(&"target".to_string()),
                "test".to_string(),
            )
            .await;
        assert!(result.is_err());

        // Test collaborative task with non-existent initiator
        let result = forest
            .execute_collaborative_task(&"nonexistent".to_string(), "test task".to_string(), vec![])
            .await;
        assert!(result.is_err());

        // Test collaborative task with non-existent involved agent
        let config = Config::new_default();
        let agent = Agent::builder("real_agent")
            .config(config)
            .build()
            .await
            .unwrap();
        forest.add_agent("real_agent".to_string(), agent).unwrap();

        let result = forest
            .execute_collaborative_task(
                &"real_agent".to_string(),
                "test task".to_string(),
                vec!["nonexistent".to_string()],
            )
            .await;
        assert!(result.is_err());
    }

    /// Tests ForestMessage creation and properties.
    #[tokio::test]
    async fn test_forest_message() {
        // Test direct message
        let msg = ForestMessage::new(
            "alice".to_string(),
            Some("bob".to_string()),
            "Hello".to_string(),
        );
        assert_eq!(msg.from, "alice");
        assert_eq!(msg.to, Some("bob".to_string()));
        assert_eq!(msg.content, "Hello");

        // Test broadcast message
        let broadcast = ForestMessage::broadcast("alice".to_string(), "Announcement".to_string());
        assert_eq!(broadcast.from, "alice");
        assert!(broadcast.to.is_none());
        assert_eq!(broadcast.content, "Announcement");

        // Test metadata
        let msg_with_meta = msg.with_metadata("priority".to_string(), "high".to_string());
        assert_eq!(
            msg_with_meta.metadata.get("priority"),
            Some(&"high".to_string())
        );
    }
}
