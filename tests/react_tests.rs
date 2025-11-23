//! # ReAct Feature Tests
//!
//! This file contains tests specifically for the ReAct (Reasoning and Acting) feature.

use helios_engine::{Agent, CalculatorTool, Config, EchoTool, LLMConfig};

/// Helper function to create a test config.
fn create_test_config() -> Config {
    Config {
        llm: LLMConfig {
            model_name: std::env::var("TEST_MODEL_NAME")
                .unwrap_or_else(|_| "gpt-3.5-turbo".to_string()),
            base_url: std::env::var("TEST_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            api_key: std::env::var("TEST_API_KEY").unwrap_or_else(|_| "test-key".to_string()),
            temperature: 0.7,
            max_tokens: 2048,
        },
        #[cfg(feature = "local")]
        local: None,
    }
}

/// Tests that an agent can be created with ReAct mode enabled.
#[tokio::test]
async fn test_react_agent_creation() {
    let config = create_test_config();

    let agent = Agent::builder("react_agent")
        .config(config)
        .react()
        .build()
        .await;

    assert!(agent.is_ok(), "Failed to create agent with ReAct mode");
    let agent = agent.unwrap();
    assert_eq!(agent.name(), "react_agent");
}

/// Tests that ReAct mode can be combined with tools.
#[tokio::test]
async fn test_react_agent_with_tools() {
    let config = create_test_config();

    let agent = Agent::builder("react_agent_with_tools")
        .config(config)
        .system_prompt("You are a helpful assistant that thinks before acting.")
        .tool(Box::new(CalculatorTool))
        .tool(Box::new(EchoTool))
        .react()
        .build()
        .await;

    assert!(agent.is_ok(), "Failed to create ReAct agent with tools");
    let agent = agent.unwrap();

    // Verify tools are registered
    let tools = agent.tool_registry().list_tools();
    assert_eq!(tools.len(), 2);
    assert!(tools.contains(&"calculator".to_string()));
    assert!(tools.contains(&"echo".to_string()));
}

/// Tests that ReAct mode can be combined with other builder options.
#[tokio::test]
async fn test_react_agent_builder_chain() {
    let config = create_test_config();

    let agent = Agent::builder("complex_react_agent")
        .config(config)
        .system_prompt("You are a reasoning assistant.")
        .tool(Box::new(CalculatorTool))
        .max_iterations(7)
        .react()
        .build()
        .await;

    assert!(agent.is_ok(), "Failed to build complex ReAct agent");
    let agent = agent.unwrap();
    assert_eq!(agent.name(), "complex_react_agent");
}

/// Tests that an agent can work without ReAct mode (default behavior).
#[tokio::test]
async fn test_agent_without_react_mode() {
    let config = create_test_config();

    let agent = Agent::builder("normal_agent")
        .config(config)
        .tool(Box::new(CalculatorTool))
        .build()
        .await;

    assert!(agent.is_ok(), "Failed to create normal agent");
    let agent = agent.unwrap();
    assert_eq!(agent.name(), "normal_agent");
}

/// Tests that ReAct can be used in any position in the builder chain.
#[tokio::test]
async fn test_react_builder_position_flexibility() {
    let config1 = create_test_config();
    let config2 = create_test_config();

    // ReAct before tools
    let agent1 = Agent::builder("react_first")
        .config(config1)
        .react()
        .tool(Box::new(CalculatorTool))
        .build()
        .await;

    // ReAct after tools
    let agent2 = Agent::builder("react_last")
        .config(config2)
        .tool(Box::new(CalculatorTool))
        .react()
        .build()
        .await;

    assert!(agent1.is_ok() && agent2.is_ok(), "Builder position matters");
}
