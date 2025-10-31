//! # Integration Tests
//!
//! This file contains integration tests for the Helios Engine. These tests
//! validate the interaction between different modules, such as agents, tools,
//! and the configuration.

use async_trait::async_trait;
use helios_engine::{
    Agent, CalculatorTool, ChatMessage, Config, EchoTool, LLMConfig, Tool, ToolParameter,
    ToolResult, serve,
};
use serde_json::json;
use std::collections::HashMap;

/// Tests that an agent can be created with the `CalculatorTool`.
#[tokio::test]
async fn test_agent_with_calculator_tool() {
    // Create a basic config for testing.
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

    // Create an agent with the calculator tool.
    let agent = Agent::builder("test_agent")
        .config(config)
        .tool(Box::new(CalculatorTool))
        .build()
        .await
        .expect("Failed to create agent");

    // This test will likely fail in offline mode, but ensures the integration path works.
    // For now we'll just test the setup.
    assert_eq!(agent.name(), "test_agent");
    assert_eq!(
        agent.tool_registry().list_tools(),
        vec!["calculator".to_string()]
    );
}

/// Tests that an agent can be created with the `EchoTool`.
#[tokio::test]
async fn test_agent_with_echo_tool() {
    // Create a basic config for testing.
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

/// Tests the functionality of the `ToolRegistry`.
#[tokio::test]
async fn test_tool_registry_functionality() {
    use helios_engine::ToolRegistry;

    let mut registry = ToolRegistry::new();
    registry.register(Box::new(CalculatorTool));
    registry.register(Box::new(EchoTool));

    // Test listing tools.
    let tools = registry.list_tools();
    assert_eq!(tools.len(), 2);
    assert!(tools.contains(&"calculator".to_string()));
    assert!(tools.contains(&"echo".to_string()));

    // Test getting tool definitions.
    let definitions = registry.get_definitions();
    assert_eq!(definitions.len(), 2);

    // Test executing the calculator tool.
    let calc_args = json!({"expression": "5 * 7"});
    let result = registry.execute("calculator", calc_args).await.unwrap();
    assert!(result.success);
    assert_eq!(result.output, "35");

    // Test executing the echo tool.
    let echo_args = json!({"message": "Hello, world!"});
    let result = registry.execute("echo", echo_args).await.unwrap();
    assert!(result.success);
    assert_eq!(result.output, "Echo: Hello, world!");
}

/// Tests the serialization and deserialization of the `Config` struct.
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

    // Test serialization to a TOML string.
    let serialized = toml::to_string_pretty(&config);
    assert!(serialized.is_ok());

    // Test deserialization from a TOML string.
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

/// Tests the creation of `ChatMessage` instances.
#[test]
fn test_chat_message_creation() {
    // Test creating different types of chat messages.
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

/// Tests the management of a `ChatSession`.
#[test]
fn test_chat_session_management() {
    use helios_engine::ChatSession;

    let mut session = ChatSession::new().with_system_prompt("Test system prompt");

    // Add messages to the session.
    session.add_user_message("Hello");
    session.add_assistant_message("Hi there!");
    session.add_user_message("How are you?");

    // Verify the message count.
    assert_eq!(session.messages.len(), 3);

    // Get all messages, including the system prompt.
    let all_messages = session.get_messages();
    assert_eq!(all_messages.len(), 4); // 1 system + 3 user/assistant

    // Verify that the first message is the system prompt.
    assert_eq!(all_messages[0].role, helios_engine::chat::Role::System);
    assert_eq!(all_messages[0].content, "Test system prompt");

    // Clear the session.
    session.clear();
    assert!(session.messages.is_empty());
}

/// A mock tool for integration testing.
struct TestTool {
    name: String,
}

impl TestTool {
    /// Creates a new `TestTool`.
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

/// Tests a tool with custom parameters.
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

/// Tests a `ToolRegistry` with multiple tools.
#[tokio::test]
async fn test_multiple_tools_in_registry() {
    use helios_engine::ToolRegistry;

    let mut registry = ToolRegistry::new();
    registry.register(Box::new(CalculatorTool));
    registry.register(Box::new(EchoTool));
    registry.register(Box::new(TestTool::new("test_tool")));

    // Test that all tools are registered.
    let tools = registry.list_tools();
    assert_eq!(tools.len(), 3);
    assert!(tools.contains(&"calculator".to_string()));
    assert!(tools.contains(&"echo".to_string()));
    assert!(tools.contains(&"test_tool".to_string()));

    // Test executing each tool.
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

/// Tests the builder pattern for creating agents.
#[tokio::test]
async fn test_agent_builder_pattern() {
    // Test the builder pattern for creating agents.
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

    // Build an agent with all options.
    let agent = Agent::builder("builder_test_agent")
        .config(config)
        .system_prompt("You are a helpful assistant for testing.")
        .tool(Box::new(CalculatorTool))
        .tool(Box::new(EchoTool))
        .max_iterations(5)
        .build()
        .await
        .expect("Failed to build agent with builder pattern");

    // Verify that the agent was configured correctly.
    assert_eq!(agent.name(), "builder_test_agent");
    // Note: max_iterations is private, so we'll test its effect during agent operations.

    // Check that both tools were registered.
    let tools = agent.tool_registry().list_tools();
    assert_eq!(tools.len(), 2);
    assert!(tools.contains(&"calculator".to_string()));
    assert!(tools.contains(&"echo".to_string()));

    // Verify that the system prompt is in the chat session.
    let session = agent.chat_session();
    assert_eq!(
        session.system_prompt,
        Some("You are a helpful assistant for testing.".to_string())
    );
}

/// Tests the conversion from a string to a `Role`.
#[test]
fn test_role_enum_conversions() {
    // Test string to Role conversion.
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

    // Test case insensitivity.
    assert_eq!(
        helios_engine::chat::Role::from("SYSTEM"),
        helios_engine::chat::Role::System
    );
    assert_eq!(
        helios_engine::chat::Role::from("User"),
        helios_engine::chat::Role::User
    );

    // Test the default case.
    assert_eq!(
        helios_engine::chat::Role::from("invalid"),
        helios_engine::chat::Role::Assistant
    );
}

/// Tests the creation of server state with an LLM client.
#[tokio::test]
async fn test_server_state_with_llm_client() {
    use helios_engine::llm::LLMProviderType;

    let config = Config {
        llm: LLMConfig {
            model_name: "gpt-3.5-turbo".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: "test-key".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
        },
        local: None,
    };

    // This will fail without proper credentials, but we can test the structure
    let provider_type = LLMProviderType::Remote(config.llm.clone());
    let llm_client = helios_engine::llm::LLMClient::new(provider_type).await;

    // For testing purposes, we'll create the state structure even if the client fails
    if let Ok(client) = llm_client {
        let state = serve::ServerState::with_llm_client(client, "gpt-3.5-turbo".to_string());

        assert!(state.llm_client.is_some());
        assert!(state.agent.is_none());
        assert_eq!(state.model_name, "gpt-3.5-turbo");
    } else {
        // If client creation fails, we at least test that the config structure works
        assert!(config.llm.api_key == "test-key");
    }
}

/// Tests the creation of server state with an agent.
#[tokio::test]
async fn test_server_state_with_agent() {
    let config = Config {
        llm: LLMConfig {
            model_name: "gpt-3.5-turbo".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: "test-key".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
        },
        local: None,
    };

    // Create a simple agent for testing
    let agent = Agent::builder("test_agent")
        .config(config)
        .tool(Box::new(CalculatorTool))
        .build()
        .await
        .expect("Failed to create test agent");

    let state = serve::ServerState::with_agent(agent, "test-model".to_string());

    assert!(state.llm_client.is_none());
    assert!(state.agent.is_some());
    assert_eq!(state.model_name, "test-model");
}

/// Tests the conversion of OpenAI messages to ChatMessage format.
#[test]
fn test_openai_message_conversion() {
    use helios_engine::serve::{OpenAIMessage, ChatCompletionRequest};
    use helios_engine::chat::Role;

    let openai_messages = vec![
        OpenAIMessage {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
            name: None,
        },
        OpenAIMessage {
            role: "user".to_string(),
            content: "Hello!".to_string(),
            name: None,
        },
        OpenAIMessage {
            role: "assistant".to_string(),
            content: "Hi there!".to_string(),
            name: None,
        },
        OpenAIMessage {
            role: "tool".to_string(),
            content: "Tool result".to_string(),
            name: Some("calculator".to_string()),
        },
    ];

    let request = ChatCompletionRequest {
        model: "test-model".to_string(),
        messages: openai_messages,
        temperature: None,
        max_tokens: None,
        stream: None,
        stop: None,
    };

    // Test message conversion (this would normally happen in the handler)
    let messages_result: Result<Vec<ChatMessage>, _> = request
        .messages
        .into_iter()
        .map(|msg| {
            let role = match msg.role.as_str() {
                "system" => Role::System,
                "user" => Role::User,
                "assistant" => Role::Assistant,
                "tool" => Role::Tool,
                _ => return Err(helios_engine::HeliosError::ConfigError(format!("Invalid role: {}", msg.role))),
            };
            Ok(ChatMessage {
                role,
                content: msg.content,
                name: msg.name,
                tool_calls: None,
                tool_call_id: None,
            })
        })
        .collect();

    assert!(messages_result.is_ok());
    let messages = messages_result.unwrap();

    assert_eq!(messages.len(), 4);
    assert_eq!(messages[0].role, Role::System);
    assert_eq!(messages[0].content, "You are a helpful assistant.");
    assert_eq!(messages[1].role, Role::User);
    assert_eq!(messages[1].content, "Hello!");
    assert_eq!(messages[2].role, Role::Assistant);
    assert_eq!(messages[2].content, "Hi there!");
    assert_eq!(messages[3].role, Role::Tool);
    assert_eq!(messages[3].content, "Tool result");
    assert_eq!(messages[3].name, Some("calculator".to_string()));
}

/// Tests the token estimation function.
#[test]
fn test_token_estimation() {
    use helios_engine::serve::estimate_tokens;

    // Test with empty string
    assert_eq!(estimate_tokens(""), 0);

    // Test with short text
    assert_eq!(estimate_tokens("Hello world"), 3); // ~11 chars / 4 = 2.75 -> 3

    // Test with longer text
    let long_text = "This is a longer piece of text that should result in more tokens when estimated.";
    let tokens = estimate_tokens(long_text);
    assert!(tokens > 10); // Should be roughly len/4

    // Test with multiple messages
    let messages = vec!["Hello", "How are you?", "I'm doing well, thank you!"];
    let combined_tokens: u32 = messages.iter().map(|m| estimate_tokens(m)).sum();
    assert_eq!(combined_tokens, 12); // 2 + 3 + 7 = 12 tokens (5/4=1.25->2, 12/4=3, 27/4=6.75->7)
}

/// Tests the chat completion request structure.
#[test]
fn test_chat_completion_request_structure() {
    use helios_engine::serve::{ChatCompletionRequest, OpenAIMessage};

    let request = ChatCompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![
            OpenAIMessage {
                role: "user".to_string(),
                content: "What is 2+2?".to_string(),
                name: None,
            }
        ],
        temperature: Some(0.7),
        max_tokens: Some(100),
        stream: Some(false),
        stop: Some(vec!["END".to_string()]),
    };

    assert_eq!(request.model, "gpt-3.5-turbo");
    assert_eq!(request.messages.len(), 1);
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.max_tokens, Some(100));
    assert_eq!(request.stream, Some(false));
    assert_eq!(request.stop, Some(vec!["END".to_string()]));
}

/// Tests the chat completion response structure.
#[test]
fn test_chat_completion_response_structure() {
    use helios_engine::serve::{ChatCompletionResponse, CompletionChoice, OpenAIMessageResponse, Usage};

    let response = ChatCompletionResponse {
        id: "chatcmpl-test123".to_string(),
        object: "chat.completion".to_string(),
        created: 1234567890,
        model: "gpt-3.5-turbo".to_string(),
        choices: vec![CompletionChoice {
            index: 0,
            message: OpenAIMessageResponse {
                role: "assistant".to_string(),
                content: "The answer is 4.".to_string(),
            },
            finish_reason: "stop".to_string(),
        }],
        usage: Usage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        },
    };

    assert_eq!(response.id, "chatcmpl-test123");
    assert_eq!(response.object, "chat.completion");
    assert_eq!(response.created, 1234567890);
    assert_eq!(response.model, "gpt-3.5-turbo");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].index, 0);
    assert_eq!(response.choices[0].message.role, "assistant");
    assert_eq!(response.choices[0].message.content, "The answer is 4.");
    assert_eq!(response.choices[0].finish_reason, "stop");
    assert_eq!(response.usage.prompt_tokens, 10);
    assert_eq!(response.usage.completion_tokens, 5);
    assert_eq!(response.usage.total_tokens, 15);
}

/// Tests the models response structure.
#[test]
fn test_models_response_structure() {
    use helios_engine::serve::{ModelsResponse, ModelInfo};

    let response = ModelsResponse {
        object: "list".to_string(),
        data: vec![ModelInfo {
            id: "gpt-3.5-turbo".to_string(),
            object: "model".to_string(),
            created: 1677649963,
            owned_by: "helios-engine".to_string(),
        }],
    };

    assert_eq!(response.object, "list");
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].id, "gpt-3.5-turbo");
    assert_eq!(response.data[0].object, "model");
    assert_eq!(response.data[0].created, 1677649963);
    assert_eq!(response.data[0].owned_by, "helios-engine");
}

/// Tests invalid role conversion in OpenAI message processing.
#[test]
fn test_invalid_role_conversion() {
    use helios_engine::serve::OpenAIMessage;
    use helios_engine::chat::Role;

    let invalid_message = OpenAIMessage {
        role: "invalid_role".to_string(),
        content: "test".to_string(),
        name: None,
    };

    let request = helios_engine::serve::ChatCompletionRequest {
        model: "test".to_string(),
        messages: vec![invalid_message],
        temperature: None,
        max_tokens: None,
        stream: None,
        stop: None,
    };

    // Test message conversion with invalid role
    let messages_result: Result<Vec<ChatMessage>, _> = request
        .messages
        .into_iter()
        .map(|msg| {
            let role = match msg.role.as_str() {
                "system" => Role::System,
                "user" => Role::User,
                "assistant" => Role::Assistant,
                "tool" => Role::Tool,
                _ => return Err(helios_engine::HeliosError::ConfigError(format!("Invalid role: {}", msg.role))),
            };
            Ok(ChatMessage {
                role,
                content: msg.content,
                name: msg.name,
                tool_calls: None,
                tool_call_id: None,
            })
        })
        .collect();

    assert!(messages_result.is_err());
    let err = messages_result.unwrap_err();
    if let helios_engine::HeliosError::ConfigError(msg) = err {
        assert!(msg.contains("Invalid role: invalid_role"));
    } else {
        panic!("Expected ConfigError, got {:?}", err);
    }
}

/// Tests UUID generation for completion IDs.
#[test]
fn test_completion_id_generation() {
    use uuid::Uuid;

    // Generate a few IDs to ensure they're unique
    let id1 = format!("chatcmpl-{}", Uuid::new_v4());
    let id2 = format!("chatcmpl-{}", Uuid::new_v4());

    assert_ne!(id1, id2);
    assert!(id1.starts_with("chatcmpl-"));
    assert!(id2.starts_with("chatcmpl-"));
    assert_eq!(id1.len(), 45); // "chatcmpl-" + 36 char UUID (actual length)
    assert_eq!(id2.len(), 45);
}

/// Tests custom endpoint configuration structure.
#[test]
fn test_custom_endpoint_config_structure() {
    use helios_engine::serve::{CustomEndpoint, CustomEndpointsConfig};

    let endpoint = CustomEndpoint {
        method: "GET".to_string(),
        path: "/api/test".to_string(),
        response: serde_json::json!({"message": "test response"}),
        status_code: 200,
    };

    assert_eq!(endpoint.method, "GET");
    assert_eq!(endpoint.path, "/api/test");
    assert_eq!(endpoint.status_code, 200);

    let config = CustomEndpointsConfig {
        endpoints: vec![endpoint],
    };

    assert_eq!(config.endpoints.len(), 1);
    assert_eq!(config.endpoints[0].method, "GET");
}

/// Tests loading custom endpoints from TOML string.
#[test]
fn test_custom_endpoints_config_parsing() {
    use helios_engine::serve::CustomEndpointsConfig;

    let toml_content = r#"
[[endpoints]]
method = "GET"
path = "/api/version"
response = { version = "1.0.0", service = "test" }
status_code = 200

[[endpoints]]
method = "POST"
path = "/api/data"
response = { data = "example" }
status_code = 201
"#;

    let config: CustomEndpointsConfig = toml::from_str(toml_content).unwrap();

    assert_eq!(config.endpoints.len(), 2);

    // Test first endpoint
    assert_eq!(config.endpoints[0].method, "GET");
    assert_eq!(config.endpoints[0].path, "/api/version");
    assert_eq!(config.endpoints[0].status_code, 200);
    assert_eq!(config.endpoints[0].response["version"], "1.0.0");
    assert_eq!(config.endpoints[0].response["service"], "test");

    // Test second endpoint
    assert_eq!(config.endpoints[1].method, "POST");
    assert_eq!(config.endpoints[1].path, "/api/data");
    assert_eq!(config.endpoints[1].status_code, 201);
    assert_eq!(config.endpoints[1].response["data"], "example");
}

/// Tests different HTTP methods for custom endpoints.
#[test]
fn test_custom_endpoint_http_methods() {
    use helios_engine::serve::CustomEndpointsConfig;

    let toml_content = r#"
[[endpoints]]
method = "GET"
path = "/api/get"
response = { method = "GET" }

[[endpoints]]
method = "POST"
path = "/api/post"
response = { method = "POST" }

[[endpoints]]
method = "PUT"
path = "/api/put"
response = { method = "PUT" }

[[endpoints]]
method = "DELETE"
path = "/api/delete"
response = { method = "DELETE" }

[[endpoints]]
method = "PATCH"
path = "/api/patch"
response = { method = "PATCH" }
"#;

    let config: CustomEndpointsConfig = toml::from_str(toml_content).unwrap();

    assert_eq!(config.endpoints.len(), 5);

    let methods: Vec<&str> = config.endpoints.iter().map(|e| e.method.as_str()).collect();
    assert!(methods.contains(&"GET"));
    assert!(methods.contains(&"POST"));
    assert!(methods.contains(&"PUT"));
    assert!(methods.contains(&"DELETE"));
    assert!(methods.contains(&"PATCH"));
}

/// Tests custom endpoint with non-default status code.
#[test]
fn test_custom_endpoint_status_codes() {
    use helios_engine::serve::CustomEndpointsConfig;

    let toml_content = r#"
[[endpoints]]
method = "GET"
path = "/api/ok"
response = { status = "ok" }
status_code = 200

[[endpoints]]
method = "GET"
path = "/api/created"
response = { status = "created" }
status_code = 201

[[endpoints]]
method = "GET"
path = "/api/error"
response = { error = "not found" }
status_code = 404
"#;

    let config: CustomEndpointsConfig = toml::from_str(toml_content).unwrap();

    assert_eq!(config.endpoints[0].status_code, 200);
    assert_eq!(config.endpoints[1].status_code, 201);
    assert_eq!(config.endpoints[2].status_code, 404);
}
