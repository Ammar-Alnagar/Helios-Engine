#![allow(dead_code)]
#![allow(unused_variables)]
use crate::chat::{ChatMessage, ChatSession};
use crate::config::Config;
use crate::error::{HeliosError, Result};
use crate::llm::{LLMClient, LLMProviderType};
use crate::tools::{ToolRegistry, ToolResult};
use serde_json::Value;

const AGENT_MEMORY_PREFIX: &str = "agent:";

pub struct Agent {
    name: String,
    llm_client: LLMClient,
    tool_registry: ToolRegistry,
    chat_session: ChatSession,
    max_iterations: usize,
}

impl Agent {
    pub async fn new(name: impl Into<String>, config: Config) -> Result<Self> {
        let provider_type = if let Some(local_config) = config.local {
            LLMProviderType::Local(local_config)
        } else {
            LLMProviderType::Remote(config.llm)
        };

        let llm_client = LLMClient::new(provider_type).await?;

        Ok(Self {
            name: name.into(),
            llm_client,
            tool_registry: ToolRegistry::new(),
            chat_session: ChatSession::new(),
            max_iterations: 10,
        })
    }

    pub fn builder(name: impl Into<String>) -> AgentBuilder {
        AgentBuilder::new(name)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.chat_session = self.chat_session.clone().with_system_prompt(prompt);
    }

    pub fn register_tool(&mut self, tool: Box<dyn crate::tools::Tool>) {
        self.tool_registry.register(tool);
    }

    pub fn tool_registry(&self) -> &ToolRegistry {
        &self.tool_registry
    }

    pub fn tool_registry_mut(&mut self) -> &mut ToolRegistry {
        &mut self.tool_registry
    }

    pub fn chat_session(&self) -> &ChatSession {
        &self.chat_session
    }

    pub fn chat_session_mut(&mut self) -> &mut ChatSession {
        &mut self.chat_session
    }

    pub fn clear_history(&mut self) {
        self.chat_session.clear();
    }

    pub async fn send_message(&mut self, message: impl Into<String>) -> Result<String> {
        let user_message = message.into();
        self.chat_session.add_user_message(user_message.clone());

        // Execute agent loop with tool calling
        let response = self.execute_with_tools().await?;

        Ok(response)
    }

    async fn execute_with_tools(&mut self) -> Result<String> {
        let mut iterations = 0;
        let tool_definitions = self.tool_registry.get_definitions();

        loop {
            if iterations >= self.max_iterations {
                return Err(HeliosError::AgentError(
                    "Maximum iterations reached".to_string(),
                ));
            }

            let messages = self.chat_session.get_messages();
            let tools_option = if tool_definitions.is_empty() {
                None
            } else {
                Some(tool_definitions.clone())
            };

            let response = self.llm_client.chat(messages, tools_option).await?;

            // Check if the response includes tool calls
            if let Some(ref tool_calls) = response.tool_calls {
                // Add assistant message with tool calls
                self.chat_session.add_message(response.clone());

                // Execute each tool call
                for tool_call in tool_calls {
                    let tool_name = &tool_call.function.name;
                    let tool_args: Value = serde_json::from_str(&tool_call.function.arguments)
                        .unwrap_or(Value::Object(serde_json::Map::new()));

                    let tool_result = self
                        .tool_registry
                        .execute(tool_name, tool_args)
                        .await
                        .unwrap_or_else(|e| {
                            ToolResult::error(format!("Tool execution failed: {}", e))
                        });

                    // Add tool result message
                    let tool_message = ChatMessage::tool(tool_result.output, tool_call.id.clone());
                    self.chat_session.add_message(tool_message);
                }

                iterations += 1;
                continue;
            }

            // No tool calls, we have the final response
            self.chat_session.add_message(response.clone());
            return Ok(response.content);
        }
    }

    pub async fn chat(&mut self, message: impl Into<String>) -> Result<String> {
        self.send_message(message).await
    }

    pub fn set_max_iterations(&mut self, max: usize) {
        self.max_iterations = max;
    }

    pub fn get_session_summary(&self) -> String {
        self.chat_session.get_summary()
    }

    pub fn clear_memory(&mut self) {
        // Only clear agent-scoped memory keys to avoid wiping general session metadata
        self.chat_session
            .metadata
            .retain(|k, _| !k.starts_with(AGENT_MEMORY_PREFIX));
    }

    #[inline]
    fn prefixed_key(key: &str) -> String {
        format!("{}{}", AGENT_MEMORY_PREFIX, key)
    }

    // Agent-scoped memory API (namespaced under "agent:")
    pub fn set_memory(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        self.chat_session
            .set_metadata(Self::prefixed_key(&key), value);
    }

    pub fn get_memory(&self, key: &str) -> Option<&String> {
        self.chat_session.get_metadata(&Self::prefixed_key(key))
    }

    pub fn remove_memory(&mut self, key: &str) -> Option<String> {
        self.chat_session.remove_metadata(&Self::prefixed_key(key))
    }

    // Convenience helpers to reduce duplication in examples
    pub fn increment_counter(&mut self, key: &str) -> u32 {
        let current = self
            .get_memory(key)
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);
        let next = current + 1;
        self.set_memory(key, next.to_string());
        next
    }

    pub fn increment_tasks_completed(&mut self) -> u32 {
        self.increment_counter("tasks_completed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::tools::{CalculatorTool, Tool, ToolParameter, ToolResult};
    use serde_json::Value;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_agent_new() {
        let config = Config::new_default();
        let agent = Agent::new("test_agent", config).await;
        assert!(agent.is_ok());
    }

    #[tokio::test]
    async fn test_agent_memory_namespacing_set_get_remove() {
        let config = Config::new_default();
        let mut agent = Agent::new("test_agent", config).await.unwrap();

        // Set and get namespaced memory
        agent.set_memory("working_directory", "/tmp");
        assert_eq!(
            agent.get_memory("working_directory"),
            Some(&"/tmp".to_string())
        );

        // Ensure underlying chat session stored the prefixed key
        assert_eq!(
            agent
                .chat_session()
                .get_metadata("agent:working_directory"),
            Some(&"/tmp".to_string())
        );
        // Non-prefixed key should not exist
        assert!(agent.chat_session().get_metadata("working_directory").is_none());

        // Remove should also be namespaced
        let removed = agent.remove_memory("working_directory");
        assert_eq!(removed.as_deref(), Some("/tmp"));
        assert!(agent.get_memory("working_directory").is_none());
    }

    #[tokio::test]
    async fn test_agent_clear_memory_scoped() {
        let config = Config::new_default();
        let mut agent = Agent::new("test_agent", config).await.unwrap();

        // Set an agent memory and a general (non-agent) session metadata key
        agent.set_memory("tasks_completed", "3");
        agent
            .chat_session_mut()
            .set_metadata("session_start", "now");

        // Clear only agent-scoped memory
        agent.clear_memory();

        // Agent memory removed
        assert!(agent.get_memory("tasks_completed").is_none());
        // General session metadata preserved
        assert_eq!(
            agent.chat_session().get_metadata("session_start"),
            Some(&"now".to_string())
        );
    }

    #[tokio::test]
    async fn test_agent_increment_helpers() {
        let config = Config::new_default();
        let mut agent = Agent::new("test_agent", config).await.unwrap();

        // tasks_completed increments from 0
        let n1 = agent.increment_tasks_completed();
        assert_eq!(n1, 1);
        assert_eq!(agent.get_memory("tasks_completed"), Some(&"1".to_string()));

        let n2 = agent.increment_tasks_completed();
        assert_eq!(n2, 2);
        assert_eq!(agent.get_memory("tasks_completed"), Some(&"2".to_string()));

        // generic counter
        let f1 = agent.increment_counter("files_accessed");
        assert_eq!(f1, 1);
        let f2 = agent.increment_counter("files_accessed");
        assert_eq!(f2, 2);
        assert_eq!(agent.get_memory("files_accessed"), Some(&"2".to_string()));
    }

    #[tokio::test]
    async fn test_agent_builder() {
        let config = Config::new_default();
        let agent = Agent::builder("test_agent")
            .config(config)
            .system_prompt("You are a helpful assistant")
            .max_iterations(5)
            .tool(Box::new(CalculatorTool))
            .build()
            .await
            .unwrap();

        assert_eq!(agent.name(), "test_agent");
        assert_eq!(agent.max_iterations, 5);
        assert_eq!(
            agent.tool_registry().list_tools(),
            vec!["calculator".to_string()]
        );
    }

    #[tokio::test]
    async fn test_agent_system_prompt() {
        let config = Config::new_default();
        let mut agent = Agent::new("test_agent", config).await.unwrap();
        agent.set_system_prompt("You are a test agent");

        // Check that the system prompt is set in chat session
        let session = agent.chat_session();
        assert_eq!(
            session.system_prompt,
            Some("You are a test agent".to_string())
        );
    }

    #[tokio::test]
    async fn test_agent_tool_registry() {
        let config = Config::new_default();
        let mut agent = Agent::new("test_agent", config).await.unwrap();

        // Initially no tools
        assert!(agent.tool_registry().list_tools().is_empty());

        // Register a tool
        agent.register_tool(Box::new(CalculatorTool));
        assert_eq!(
            agent.tool_registry().list_tools(),
            vec!["calculator".to_string()]
        );
    }

    #[tokio::test]
    async fn test_agent_clear_history() {
        let config = Config::new_default();
        let mut agent = Agent::new("test_agent", config).await.unwrap();

        // Add a message to the chat session
        agent.chat_session_mut().add_user_message("Hello");
        assert!(!agent.chat_session().messages.is_empty());

        // Clear history
        agent.clear_history();
        assert!(agent.chat_session().messages.is_empty());
    }

    // Mock tool for testing
    struct MockTool;

    #[async_trait::async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            "mock_tool"
        }

        fn description(&self) -> &str {
            "A mock tool for testing"
        }

        fn parameters(&self) -> HashMap<String, ToolParameter> {
            let mut params = HashMap::new();
            params.insert(
                "input".to_string(),
                ToolParameter {
                    param_type: "string".to_string(),
                    description: "Input parameter".to_string(),
                    required: Some(true),
                },
            );
            params
        }

        async fn execute(&self, args: Value) -> crate::Result<ToolResult> {
            let input = args
                .get("input")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            Ok(ToolResult::success(format!("Mock tool output: {}", input)))
        }
    }
}

pub struct AgentBuilder {
    name: String,
    config: Option<Config>,
    system_prompt: Option<String>,
    tools: Vec<Box<dyn crate::tools::Tool>>,
    max_iterations: usize,
}

impl AgentBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            config: None,
            system_prompt: None,
            tools: Vec::new(),
            max_iterations: 10,
        }
    }

    pub fn config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn tool(mut self, tool: Box<dyn crate::tools::Tool>) -> Self {
        self.tools.push(tool);
        self
    }

    pub fn max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    pub async fn build(self) -> Result<Agent> {
        let config = self
            .config
            .ok_or_else(|| HeliosError::AgentError("Config is required".to_string()))?;

        let mut agent = Agent::new(self.name, config).await?;

        if let Some(prompt) = self.system_prompt {
            agent.set_system_prompt(prompt);
        }

        for tool in self.tools {
            agent.register_tool(tool);
        }

        agent.set_max_iterations(self.max_iterations);

        Ok(agent)
    }
}
