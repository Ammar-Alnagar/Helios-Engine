//! # Agent Module
//!
//! This module defines the `Agent` struct, the core of the Helios Engine. Agents are
//! autonomous entities that can interact with users, use tools, and manage their
//! own chat history. The `AgentBuilder` provides a convenient way to construct
//! and configure agents.

#![allow(dead_code)]
#![allow(unused_variables)]
use crate::chat::{ChatMessage, ChatSession};
use crate::config::Config;
use crate::error::{HeliosError, Result};
use crate::llm::{LLMClient, LLMProviderType};
use crate::tools::{ToolRegistry, ToolResult};
use serde_json::Value;

/// Prefix for agent-specific keys in the chat session metadata.
const AGENT_MEMORY_PREFIX: &str = "agent:";

/// Represents an LLM-powered agent that can chat, use tools, and manage a conversation.
pub struct Agent {
    /// The name of the agent.
    name: String,
    /// The client for interacting with the Large Language Model.
    llm_client: LLMClient,
    /// The registry of tools available to the agent.
    tool_registry: ToolRegistry,
    /// The chat session, which stores the conversation history.
    chat_session: ChatSession,
    /// The maximum number of iterations for tool execution in a single turn.
    max_iterations: usize,
    /// Whether the agent uses ReAct mode (Reasoning and Acting).
    react_mode: bool,
}

impl Agent {
    /// Creates a new agent with the given name and configuration.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the agent.
    /// * `config` - The configuration for the agent.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `Agent` instance.
    async fn new(name: impl Into<String>, config: Config) -> Result<Self> {
        #[cfg(feature = "local")]
        let provider_type = if let Some(local_config) = config.local {
            LLMProviderType::Local(local_config)
        } else {
            LLMProviderType::Remote(config.llm)
        };

        #[cfg(not(feature = "local"))]
        let provider_type = LLMProviderType::Remote(config.llm);

        let llm_client = LLMClient::new(provider_type).await?;

        Ok(Self {
            name: name.into(),
            llm_client,
            tool_registry: ToolRegistry::new(),
            chat_session: ChatSession::new(),
            max_iterations: 10,
            react_mode: false,
        })
    }

    /// Returns a new `AgentBuilder` for constructing an agent.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the agent.
    pub fn builder(name: impl Into<String>) -> AgentBuilder {
        AgentBuilder::new(name)
    }

    /// Returns the name of the agent.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the system prompt for the agent.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The system prompt to set.
    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.chat_session = self.chat_session.clone().with_system_prompt(prompt);
    }

    /// Registers a tool with the agent.
    ///
    /// # Arguments
    ///
    /// * `tool` - The tool to register.
    pub fn register_tool(&mut self, tool: Box<dyn crate::tools::Tool>) {
        self.tool_registry.register(tool);
    }

    /// Returns a reference to the agent's tool registry.
    pub fn tool_registry(&self) -> &ToolRegistry {
        &self.tool_registry
    }

    /// Returns a mutable reference to the agent's tool registry.
    pub fn tool_registry_mut(&mut self) -> &mut ToolRegistry {
        &mut self.tool_registry
    }

    /// Returns a reference to the agent's chat session.
    pub fn chat_session(&self) -> &ChatSession {
        &self.chat_session
    }

    /// Returns a mutable reference to the agent's chat session.
    pub fn chat_session_mut(&mut self) -> &mut ChatSession {
        &mut self.chat_session
    }

    /// Clears the agent's chat history.
    pub fn clear_history(&mut self) {
        self.chat_session.clear();
    }

    /// Sends a message to the agent and gets a response.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send.
    ///
    /// # Returns
    ///
    /// A `Result` containing the agent's response.
    pub async fn send_message(&mut self, message: impl Into<String>) -> Result<String> {
        let user_message = message.into();
        self.chat_session.add_user_message(user_message.clone());

        // Execute agent loop with tool calling
        let response = self.execute_with_tools().await?;

        Ok(response)
    }

    /// Generates reasoning for the current task in ReAct mode.
    async fn generate_reasoning(&mut self) -> Result<String> {
        let reasoning_prompt = r#"Before taking any action, think through this step by step:

1. What is the user asking for?
2. What information or tools do I need to answer this?
3. What is my plan to solve this problem?

Provide your reasoning in a clear, structured way."#;

        // Create a temporary reasoning message
        let mut reasoning_messages = self.chat_session.get_messages();
        reasoning_messages.push(ChatMessage::user(reasoning_prompt));

        // Get reasoning from LLM without tools
        let response = self
            .llm_client
            .chat(reasoning_messages, None, None, None, None)
            .await?;

        // Store reasoning in chat session as a system-like note
        self.chat_session.add_message(ChatMessage::user(format!(
            "[Internal Reasoning]\n{}",
            response.content
        )));
        self.chat_session.add_message(ChatMessage::assistant(
            "[Reasoning complete, proceeding with action]",
        ));

        Ok(response.content)
    }

    /// Executes the agent's main loop, including tool calls.
    async fn execute_with_tools(&mut self) -> Result<String> {
        self.execute_with_tools_streaming().await
    }

    /// Executes the agent's main loop with streaming, including tool calls.
    async fn execute_with_tools_streaming(&mut self) -> Result<String> {
        self.execute_with_tools_streaming_with_params(None, None, None)
            .await
    }

    /// Executes the agent's main loop with parameters, including tool calls.
    async fn execute_with_tools_with_params(
        &mut self,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        stop: Option<Vec<String>>,
    ) -> Result<String> {
        // If ReAct mode is enabled, generate reasoning first
        if self.react_mode && !self.tool_registry.get_definitions().is_empty() {
            let reasoning = self.generate_reasoning().await?;
            println!("\n💭 ReAct Reasoning:\n{}\n", reasoning);
        }

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

            let response = self
                .llm_client
                .chat(
                    messages,
                    tools_option,
                    temperature,
                    max_tokens,
                    stop.clone(),
                )
                .await?;

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

    /// Executes the agent's main loop with parameters and streaming, including tool calls.
    async fn execute_with_tools_streaming_with_params(
        &mut self,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        stop: Option<Vec<String>>,
    ) -> Result<String> {
        // If ReAct mode is enabled, generate reasoning first
        if self.react_mode && !self.tool_registry.get_definitions().is_empty() {
            let reasoning = self.generate_reasoning().await?;
            println!("\n💭 ReAct Reasoning:\n{}\n", reasoning);
        }

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

            let mut streamed_content = String::new();

            let stream_result = self
                .llm_client
                .chat_stream(
                    messages,
                    tools_option, // Enable tools for streaming
                    temperature,
                    max_tokens,
                    stop.clone(),
                    |chunk| {
                        // Print chunk to stdout for visible streaming
                        print!("{}", chunk);
                        let _ = std::io::Write::flush(&mut std::io::stdout());
                        streamed_content.push_str(chunk);
                    },
                )
                .await;

            let response = stream_result?;

            // Print newline after streaming completes
            println!();

            // Check if the response includes tool calls
            if let Some(ref tool_calls) = response.tool_calls {
                // Add assistant message with tool calls
                let mut msg_with_content = response.clone();
                msg_with_content.content = streamed_content.clone();
                self.chat_session.add_message(msg_with_content);

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

            // No tool calls, we have the final response with streamed content
            let mut final_msg = response;
            final_msg.content = streamed_content.clone();
            self.chat_session.add_message(final_msg);
            return Ok(streamed_content);
        }
    }

    /// A convenience method for sending a message to the agent.
    pub async fn chat(&mut self, message: impl Into<String>) -> Result<String> {
        self.send_message(message).await
    }

    /// Sets the maximum number of iterations for tool execution.
    ///
    /// # Arguments
    ///
    /// * `max` - The maximum number of iterations.
    pub fn set_max_iterations(&mut self, max: usize) {
        self.max_iterations = max;
    }

    /// Returns a summary of the current chat session.
    pub fn get_session_summary(&self) -> String {
        self.chat_session.get_summary()
    }

    /// Clears the agent's memory (agent-scoped metadata).
    pub fn clear_memory(&mut self) {
        // Only clear agent-scoped memory keys to avoid wiping general session metadata
        self.chat_session
            .metadata
            .retain(|k, _| !k.starts_with(AGENT_MEMORY_PREFIX));
    }

    /// Prefixes a key with the agent memory prefix.
    #[inline]
    fn prefixed_key(key: &str) -> String {
        format!("{}{}", AGENT_MEMORY_PREFIX, key)
    }

    // Agent-scoped memory API (namespaced under "agent:")
    /// Sets a value in the agent's memory.
    pub fn set_memory(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        self.chat_session
            .set_metadata(Self::prefixed_key(&key), value);
    }

    /// Gets a value from the agent's memory.
    pub fn get_memory(&self, key: &str) -> Option<&String> {
        self.chat_session.get_metadata(&Self::prefixed_key(key))
    }

    /// Removes a value from the agent's memory.
    pub fn remove_memory(&mut self, key: &str) -> Option<String> {
        self.chat_session.remove_metadata(&Self::prefixed_key(key))
    }

    // Convenience helpers to reduce duplication in examples
    /// Increments a counter in the agent's memory.
    pub fn increment_counter(&mut self, key: &str) -> u32 {
        let current = self
            .get_memory(key)
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);
        let next = current + 1;
        self.set_memory(key, next.to_string());
        next
    }

    /// Increments the "tasks_completed" counter in the agent's memory.
    pub fn increment_tasks_completed(&mut self) -> u32 {
        self.increment_counter("tasks_completed")
    }

    /// Executes a stateless conversation with the provided message history.
    ///
    /// This method creates a temporary chat session with the provided messages
    /// and executes the agent logic without modifying the agent's persistent session.
    /// This is useful for OpenAI API compatibility where each request contains
    /// the full conversation history.
    ///
    /// # Arguments
    ///
    /// * `messages` - The complete conversation history for this request
    /// * `temperature` - Optional temperature parameter for generation
    /// * `max_tokens` - Optional maximum tokens parameter for generation
    /// * `stop` - Optional stop sequences for generation
    ///
    /// # Returns
    ///
    /// A `Result` containing the assistant's response content.
    pub async fn chat_with_history(
        &mut self,
        messages: Vec<ChatMessage>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        stop: Option<Vec<String>>,
    ) -> Result<String> {
        // Create a temporary session with the provided messages
        let mut temp_session = ChatSession::new();

        // Add all messages to the temporary session
        for message in messages {
            temp_session.add_message(message);
        }

        // Execute agent loop with tool calling using the temporary session
        self.execute_with_tools_temp_session(temp_session, temperature, max_tokens, stop)
            .await
    }

    /// Executes the agent's main loop with a temporary session, including tool calls.
    async fn execute_with_tools_temp_session(
        &mut self,
        mut temp_session: ChatSession,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        stop: Option<Vec<String>>,
    ) -> Result<String> {
        let mut iterations = 0;
        let tool_definitions = self.tool_registry.get_definitions();

        loop {
            if iterations >= self.max_iterations {
                return Err(HeliosError::AgentError(
                    "Maximum iterations reached".to_string(),
                ));
            }

            let messages = temp_session.get_messages();
            let tools_option = if tool_definitions.is_empty() {
                None
            } else {
                Some(tool_definitions.clone())
            };

            let response = self
                .llm_client
                .chat(
                    messages,
                    tools_option,
                    temperature,
                    max_tokens,
                    stop.clone(),
                )
                .await?;

            // Check if the response includes tool calls
            if let Some(ref tool_calls) = response.tool_calls {
                // Add assistant message with tool calls to temp session
                temp_session.add_message(response.clone());

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

                    // Add tool result message to temp session
                    let tool_message = ChatMessage::tool(tool_result.output, tool_call.id.clone());
                    temp_session.add_message(tool_message);
                }

                iterations += 1;
                continue;
            }

            // No tool calls, we have the final response
            return Ok(response.content);
        }
    }

    /// Executes a stateless conversation with the provided message history and streams the response.
    ///
    /// This method creates a temporary chat session with the provided messages
    /// and streams the agent's response in real-time as tokens are generated.
    /// Note: Tool calls are not supported in streaming mode yet - they will be
    /// executed after the initial response is complete.
    ///
    /// # Arguments
    ///
    /// * `messages` - The complete conversation history for this request
    /// * `temperature` - Optional temperature parameter for generation
    /// * `max_tokens` - Optional maximum tokens parameter for generation
    /// * `stop` - Optional stop sequences for generation
    /// * `on_chunk` - Callback function called for each chunk of generated text
    ///
    /// # Returns
    ///
    /// A `Result` containing the final assistant message after streaming is complete.
    pub async fn chat_stream_with_history<F>(
        &mut self,
        messages: Vec<ChatMessage>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        stop: Option<Vec<String>>,
        on_chunk: F,
    ) -> Result<ChatMessage>
    where
        F: FnMut(&str) + Send,
    {
        // Create a temporary session with the provided messages
        let mut temp_session = ChatSession::new();

        // Add all messages to the temporary session
        for message in messages {
            temp_session.add_message(message);
        }

        // For now, use streaming for the initial response only
        // Tool calls will be handled after the stream completes
        self.execute_streaming_with_tools_temp_session(
            temp_session,
            temperature,
            max_tokens,
            stop,
            on_chunk,
        )
        .await
    }

    /// Executes the agent's main loop with streaming and a temporary session.
    async fn execute_streaming_with_tools_temp_session<F>(
        &mut self,
        mut temp_session: ChatSession,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        stop: Option<Vec<String>>,
        mut on_chunk: F,
    ) -> Result<ChatMessage>
    where
        F: FnMut(&str) + Send,
    {
        let mut iterations = 0;
        let tool_definitions = self.tool_registry.get_definitions();

        loop {
            if iterations >= self.max_iterations {
                return Err(HeliosError::AgentError(
                    "Maximum iterations reached".to_string(),
                ));
            }

            let messages = temp_session.get_messages();
            let tools_option = if tool_definitions.is_empty() {
                None
            } else {
                Some(tool_definitions.clone())
            };

            // Use streaming for all iterations
            let mut streamed_content = String::new();

            let stream_result = self
                .llm_client
                .chat_stream(
                    messages,
                    tools_option,
                    temperature,
                    max_tokens,
                    stop.clone(),
                    |chunk| {
                        on_chunk(chunk);
                        streamed_content.push_str(chunk);
                    },
                )
                .await;

            match stream_result {
                Ok(response) => {
                    // Check if the response includes tool calls
                    if let Some(ref tool_calls) = response.tool_calls {
                        // Add assistant message with tool calls to temp session
                        let mut msg_with_content = response.clone();
                        msg_with_content.content = streamed_content.clone();
                        temp_session.add_message(msg_with_content);

                        // Execute each tool call
                        for tool_call in tool_calls {
                            let tool_name = &tool_call.function.name;
                            let tool_args: Value =
                                serde_json::from_str(&tool_call.function.arguments)
                                    .unwrap_or(Value::Object(serde_json::Map::new()));

                            let tool_result = self
                                .tool_registry
                                .execute(tool_name, tool_args)
                                .await
                                .unwrap_or_else(|e| {
                                    ToolResult::error(format!("Tool execution failed: {}", e))
                                });

                            // Add tool result message to temp session
                            let tool_message =
                                ChatMessage::tool(tool_result.output, tool_call.id.clone());
                            temp_session.add_message(tool_message);
                        }

                        iterations += 1;
                        continue; // Continue the loop for another iteration
                    } else {
                        // No tool calls, return the final response with streamed content
                        let mut final_msg = response;
                        final_msg.content = streamed_content;
                        return Ok(final_msg);
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }
}

pub struct AgentBuilder {
    name: String,
    config: Option<Config>,
    system_prompt: Option<String>,
    tools: Vec<Box<dyn crate::tools::Tool>>,
    max_iterations: usize,
    react_mode: bool,
}

impl AgentBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            config: None,
            system_prompt: None,
            tools: Vec::new(),
            max_iterations: 10,
            react_mode: false,
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

    /// Adds a single tool to the agent.
    pub fn tool(mut self, tool: Box<dyn crate::tools::Tool>) -> Self {
        self.tools.push(tool);
        self
    }

    /// Adds multiple tools to the agent at once.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use helios_engine::{Agent, Config, CalculatorTool, EchoTool};
    /// # async fn example() -> helios_engine::Result<()> {
    /// # let config = Config::new_default();
    /// let agent = Agent::builder("MyAgent")
    ///     .config(config)
    ///     .tools(vec![
    ///         Box::new(CalculatorTool),
    ///         Box::new(EchoTool),
    ///     ])
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn tools(mut self, tools: Vec<Box<dyn crate::tools::Tool>>) -> Self {
        self.tools.extend(tools);
        self
    }

    pub fn max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Enables ReAct mode for the agent.
    ///
    /// In ReAct mode, the agent will reason about the task and create a plan
    /// before taking actions. This helps the agent think through problems
    /// more systematically and make better decisions.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use helios_engine::{Agent, Config};
    /// # async fn example() -> helios_engine::Result<()> {
    /// # let config = Config::new_default();
    /// let agent = Agent::builder("MyAgent")
    ///     .config(config)
    ///     .react()
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn react(mut self) -> Self {
        self.react_mode = true;
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
        agent.react_mode = self.react_mode;

        Ok(agent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::tools::{CalculatorTool, Tool, ToolParameter, ToolResult};
    use serde_json::Value;
    use std::collections::HashMap;

    /// Tests that an agent can be created using the builder.
    #[tokio::test]
    async fn test_agent_creation_via_builder() {
        let config = Config::new_default();
        let agent = Agent::builder("test_agent").config(config).build().await;
        assert!(agent.is_ok());
    }

    /// Tests the namespacing of agent memory.
    #[tokio::test]
    async fn test_agent_memory_namespacing_set_get_remove() {
        let config = Config::new_default();
        let mut agent = Agent::builder("test_agent")
            .config(config)
            .build()
            .await
            .unwrap();

        // Set and get namespaced memory
        agent.set_memory("working_directory", "/tmp");
        assert_eq!(
            agent.get_memory("working_directory"),
            Some(&"/tmp".to_string())
        );

        // Ensure underlying chat session stored the prefixed key
        assert_eq!(
            agent.chat_session().get_metadata("agent:working_directory"),
            Some(&"/tmp".to_string())
        );
        // Non-prefixed key should not exist
        assert!(agent
            .chat_session()
            .get_metadata("working_directory")
            .is_none());

        // Remove should also be namespaced
        let removed = agent.remove_memory("working_directory");
        assert_eq!(removed.as_deref(), Some("/tmp"));
        assert!(agent.get_memory("working_directory").is_none());
    }

    /// Tests that clearing agent memory only affects agent-scoped data.
    #[tokio::test]
    async fn test_agent_clear_memory_scoped() {
        let config = Config::new_default();
        let mut agent = Agent::builder("test_agent")
            .config(config)
            .build()
            .await
            .unwrap();

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

    /// Tests the increment helper methods for agent memory.
    #[tokio::test]
    async fn test_agent_increment_helpers() {
        let config = Config::new_default();
        let mut agent = Agent::builder("test_agent")
            .config(config)
            .build()
            .await
            .unwrap();

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

    /// Tests the full functionality of the agent builder.
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

    /// Tests setting the system prompt for an agent.
    #[tokio::test]
    async fn test_agent_system_prompt() {
        let config = Config::new_default();
        let mut agent = Agent::builder("test_agent")
            .config(config)
            .build()
            .await
            .unwrap();
        agent.set_system_prompt("You are a test agent");

        // Check that the system prompt is set in chat session
        let session = agent.chat_session();
        assert_eq!(
            session.system_prompt,
            Some("You are a test agent".to_string())
        );
    }

    /// Tests the tool registry functionality of an agent.
    #[tokio::test]
    async fn test_agent_tool_registry() {
        let config = Config::new_default();
        let mut agent = Agent::builder("test_agent")
            .config(config)
            .build()
            .await
            .unwrap();

        // Initially no tools
        assert!(agent.tool_registry().list_tools().is_empty());

        // Register a tool
        agent.register_tool(Box::new(CalculatorTool));
        assert_eq!(
            agent.tool_registry().list_tools(),
            vec!["calculator".to_string()]
        );
    }

    /// Tests clearing the chat history of an agent.
    #[tokio::test]
    async fn test_agent_clear_history() {
        let config = Config::new_default();
        let mut agent = Agent::builder("test_agent")
            .config(config)
            .build()
            .await
            .unwrap();

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
