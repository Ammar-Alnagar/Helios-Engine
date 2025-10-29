use crate::chat::{ChatMessage, ChatSession};
use crate::config::Config;
use crate::error::{HeliosError, Result};
use crate::llm::{LLMClient, LLMProviderType};
use crate::tools::{ToolRegistry, ToolResult};
use serde_json::Value;

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
                    let tool_message = ChatMessage::tool(
                        tool_result.output,
                        tool_call.id.clone(),
                    );
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
