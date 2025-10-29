//! Integration tests for the Helios Engine
//! These tests validate the interaction between different modules

use async_trait::async_trait;
use helios_engine::{
    Agent, CalculatorTool, ChatMessage, Config, EchoTool, LLMConfig, Tool, ToolParameter,
    ToolResult,
};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_agent_with_calculator_tool() {
    // Create a basic config for testing
    let config = Config {
        llm: LLMConfig {
            model_name: std::env::var("TEST_MODEL_NAME")
                .unwrap_or_else(|_| "gpt-3.5-turbo".to_string()),
            base_url: std::env::var("TEST_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            api_key: std::env::var("TEST_API_KEY").unwrap_or_else(|_| "test-key".to_string()),
            temperature: 0.7,
            max_tokens: 2048,
        },
        local: None,
    };

    // Create an agent with calculator tool
    let agent = Agent::builder("test_agent")
        .config(config)
        .tool(Box::new(CalculatorTool))
        .build()
        .await
        .expect("Failed to create agent");

    // This test will likely fail in offline mode, but ensures the integration path works
    // For now we'll just test the setup
    assert_eq!(agent.name(), "test_agent");
    assert_eq!(
        agent.tool_registry().list_tools(),
        vec!["calculator".to_string()]
    );
}

#[tokio::test]
async fn test_agent_with_echo_tool() {
    // Create a basic config for testing
    let config = Config {
        llm: LLMConfig {
            model_name: std::env::var("TEST_MODEL_NAME")
                .unwrap_or_else(|_| "gpt-3.5-turbo".to_string()),
            base_url: std::env::var("TEST_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            api_key: std::env::var("TEST_API_KEY").unwrap_or_else(|_| "test-key".to_string()),
            temperature: 0.7,
            max_tokens: 2048,
        },
        local: None,
    };

    let agent = Agent::builder("echo_test_agent")
        .config(config)
        .tool(Box::new(EchoTool))
        .build()
        .await
        .expect("Failed to create agent");

    assert_eq!(agent.name(), "echo_test_agent");
    assert_eq!(agent.tool_registry().list_tools(), vec!["echo".to_string()]);
}

#[tokio::test]
async fn test_tool_registry_functionality() {
    use helios_engine::ToolRegistry;

    let mut registry = ToolRegistry::new();
    registry.register(Box::new(CalculatorTool));
    registry.register(Box::new(EchoTool));

    // Test listing tools
    let tools = registry.list_tools();
    assert_eq!(tools.len(), 2);
    assert!(tools.contains(&"calculator".to_string()));
    assert!(tools.contains(&"echo".to_string()));

    // Test get definitions
    let definitions = registry.get_definitions();
    assert_eq!(definitions.len(), 2);

    // Test executing calculator tool
    let calc_args = json!({"expression": "5 * 7"});
    let result = registry.execute("calculator", calc_args).await.unwrap();
    assert!(result.success);
    assert_eq!(result.output, "35");

    // Test executing echo tool
    let echo_args = json!({"message": "Hello, world!"});
    let result = registry.execute("echo", echo_args).await.unwrap();
    assert!(result.success);
    assert_eq!(result.output, "Echo: Hello, world!");
}

#[test]
fn test_config_serialization() {
    use helios_engine::LocalConfig;

    let config = Config {
        llm: LLMConfig {
            model_name: "gpt-3.5-turbo".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: "test-key".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
        },
        local: Some(LocalConfig {
            huggingface_repo: "test/repo".to_string(),
            model_file: "model.gguf".to_string(),
            context_size: 2048,
            temperature: 0.7,
            max_tokens: 2048,
        }),
    };

    // Test serialization
    let serialized = toml::to_string_pretty(&config);
    assert!(serialized.is_ok());

    // Test deserialization
    let config_str = r#"
[llm]
model_name = "gpt-4"
base_url = "https://api.openai.com/v1"
api_key = "test-key-2"
temperature = 0.5
max_tokens = 1024

[local]
huggingface_repo = "microsoft/Phi-3-mini-4k-instruct-gguf"
model_file = "Phi-3-mini-4k-instruct-q4.gguf"
context_size = 4096
temperature = 0.5
max_tokens = 1024
"#;

    let parsed_config: Result<Config, _> = toml::from_str(config_str);
    assert!(parsed_config.is_ok());

    let parsed = parsed_config.unwrap();
    assert_eq!(parsed.llm.model_name, "gpt-4");
    assert_eq!(parsed.llm.temperature, 0.5);
    assert!(parsed.local.is_some());
    assert_eq!(
        parsed.local.as_ref().unwrap().huggingface_repo,
        "microsoft/Phi-3-mini-4k-instruct-gguf"
    );
}

#[test]
fn test_chat_message_creation() {
    // Test creating different types of chat messages
    let system_msg = ChatMessage::system("System prompt");
    assert_eq!(system_msg.role, helios_engine::chat::Role::System);
    assert_eq!(system_msg.content, "System prompt");

    let user_msg = ChatMessage::user("User message");
    assert_eq!(user_msg.role, helios_engine::chat::Role::User);
    assert_eq!(user_msg.content, "User message");

    let assistant_msg = ChatMessage::assistant("Assistant response");
    assert_eq!(assistant_msg.role, helios_engine::chat::Role::Assistant);
    assert_eq!(assistant_msg.content, "Assistant response");

    let tool_msg = ChatMessage::tool("Tool result", "call_123");
    assert_eq!(tool_msg.role, helios_engine::chat::Role::Tool);
    assert_eq!(tool_msg.content, "Tool result");
    assert_eq!(tool_msg.tool_call_id, Some("call_123".to_string()));
}

#[test]
fn test_chat_session_management() {
    use helios_engine::ChatSession;

    let mut session = ChatSession::new().with_system_prompt("Test system prompt");

    // Add messages
    session.add_user_message("Hello");
    session.add_assistant_message("Hi there!");
    session.add_user_message("How are you?");

    // Verify message count
    assert_eq!(session.messages.len(), 3);

    // Get all messages including system
    let all_messages = session.get_messages();
    assert_eq!(all_messages.len(), 4); // 1 system + 3 user/assistant

    // Verify first message is system
    assert_eq!(all_messages[0].role, helios_engine::chat::Role::System);
    assert_eq!(all_messages[0].content, "Test system prompt");

    // Clear session
    session.clear();
    assert!(session.messages.is_empty());
}

// Mock tool for testing complex integration
struct TestTool {
    name: String,
}

impl TestTool {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl Tool for TestTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "A test tool for integration testing"
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "input".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Input string".to_string(),
                required: Some(true),
            },
        );
        params
    }

    async fn execute(&self, args: serde_json::Value) -> helios_engine::Result<ToolResult> {
        let input = args
            .get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        Ok(ToolResult::success(format!("Processed: {}", input)))
    }
}

#[tokio::test]
async fn test_tool_with_custom_parameters() {
    let test_tool = TestTool::new("test_tool");
    assert_eq!(test_tool.name(), "test_tool");
    assert_eq!(
        test_tool.description(),
        "A test tool for integration testing"
    );

    let params = test_tool.parameters();
    assert!(params.contains_key("input"));

    let args = json!({"input": "test value"});
    let result = test_tool.execute(args).await.unwrap();
    assert!(result.success);
    assert_eq!(result.output, "Processed: test value");
}

#[tokio::test]
async fn test_multiple_tools_in_registry() {
    use helios_engine::ToolRegistry;

    let mut registry = ToolRegistry::new();
    registry.register(Box::new(CalculatorTool));
    registry.register(Box::new(EchoTool));
    registry.register(Box::new(TestTool::new("test_tool")));

    // Test all tools are registered
    let tools = registry.list_tools();
    assert_eq!(tools.len(), 3);
    assert!(tools.contains(&"calculator".to_string()));
    assert!(tools.contains(&"echo".to_string()));
    assert!(tools.contains(&"test_tool".to_string()));

    // Test executing each tool
    let calc_result = registry
        .execute("calculator", json!({"expression": "10 + 5"}))
        .await
        .unwrap();
    assert!(calc_result.success);
    assert_eq!(calc_result.output, "15");

    let echo_result = registry
        .execute("echo", json!({"message": "Hello"}))
        .await
        .unwrap();
    assert!(echo_result.success);
    assert_eq!(echo_result.output, "Echo: Hello");

    let test_result = registry
        .execute("test_tool", json!({"input": "integration test"}))
        .await
        .unwrap();
    assert!(test_result.success);
    assert_eq!(test_result.output, "Processed: integration test");
}

#[tokio::test]
async fn test_agent_builder_pattern() {
    // Test the builder pattern for creating agents
    let config = Config {
        llm: LLMConfig {
            model_name: std::env::var("TEST_MODEL_NAME")
                .unwrap_or_else(|_| "gpt-3.5-turbo".to_string()),
            base_url: std::env::var("TEST_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            api_key: std::env::var("TEST_API_KEY").unwrap_or_else(|_| "test-key".to_string()),
            temperature: 0.7,
            max_tokens: 2048,
        },
        local: None,
    };

    // Build an agent with all options
    let agent = Agent::builder("builder_test_agent")
        .config(config)
        .system_prompt("You are a helpful assistant for testing.")
        .tool(Box::new(CalculatorTool))
        .tool(Box::new(EchoTool))
        .max_iterations(5)
        .build()
        .await
        .expect("Failed to build agent with builder pattern");

    // Verify the agent was configured correctly
    assert_eq!(agent.name(), "builder_test_agent");
    // Note: max_iterations is private, so we'll test its effect during agent operations

    // Check that both tools were registered
    let tools = agent.tool_registry().list_tools();
    assert_eq!(tools.len(), 2);
    assert!(tools.contains(&"calculator".to_string()));
    assert!(tools.contains(&"echo".to_string()));

    // Verify the system prompt is in the chat session
    let session = agent.chat_session();
    assert_eq!(
        session.system_prompt,
        Some("You are a helpful assistant for testing.".to_string())
    );
}

#[test]
fn test_role_enum_conversions() {
    // Test string to Role conversion
    assert_eq!(
        helios_engine::chat::Role::from("system"),
        helios_engine::chat::Role::System
    );
    assert_eq!(
        helios_engine::chat::Role::from("user"),
        helios_engine::chat::Role::User
    );
    assert_eq!(
        helios_engine::chat::Role::from("assistant"),
        helios_engine::chat::Role::Assistant
    );
    assert_eq!(
        helios_engine::chat::Role::from("tool"),
        helios_engine::chat::Role::Tool
    );

    // Test case insensitivity
    assert_eq!(
        helios_engine::chat::Role::from("SYSTEM"),
        helios_engine::chat::Role::System
    );
    assert_eq!(
        helios_engine::chat::Role::from("User"),
        helios_engine::chat::Role::User
    );

    // Test default case
    assert_eq!(
        helios_engine::chat::Role::from("invalid"),
        helios_engine::chat::Role::Assistant
    );
}
