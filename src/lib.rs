//! # Helios Engine
//!
//! Helios is a powerful and flexible Rust framework for building LLM-powered agents
//! with tool support, chat capabilities, and easy configuration management.
//!
//! ## Quick Start
//!
//! ### Using as a Library (Direct LLM Calls)
//!
//! ## Example
//! ```no_run
//! use helios_engine::{LLMClient, ChatMessage};
//! use helios_engine::config::LLMConfig;
//!
//! #[tokio::main]
//! async fn main() -> helios_engine::Result<()> {
//!     let llm_config = LLMConfig {
//!         model_name: "gpt-3.5-turbo".to_string(),
//!         base_url: "https://api.openai.com/v1".to_string(),
//!         api_key: std::env::var("OPENAI_API_KEY").unwrap(),
//!         temperature: 0.7,
//!         max_tokens: 2048,
//!     };
//!
//!     let client = LLMClient::new(helios_engine::llm::LLMProviderType::Remote(llm_config)).await?;
//!     let messages = vec![
//!         ChatMessage::system("You are a helpful assistant."),
//!         ChatMessage::user("What is the capital of France?"),
//!     ];
//!
//!     let response = client.chat(messages, None).await?;
//!     println!("Response: {}", response.content);
//!     Ok(())
//! }
//! ```
//!
//! ## Overview
//!
//! ```no_run
//! use helios_engine::{Agent, Config, CalculatorTool};
//!
//! #[tokio::main]
//! async fn main() -> helios_engine::Result<()> {
//!     let config = Config::from_file("config.toml")?;
//!
//!     let mut agent = Agent::builder("MyAgent")
//!         .config(config)
//!         .system_prompt("You are a helpful assistant.")
//!         .tool(Box::new(CalculatorTool))
//!         .build()
//!         .await?;
//!
//!     let response = agent.chat("What is 15 * 7?").await?;
//!     println!("Agent: {}", response);
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **Direct LLM Access**: Use `LLMClient` for simple, direct calls to LLM models.
//! - **Agent System**: Create intelligent agents with tools and persistent conversation.
//! - **Tool Support**: Extensible tool system for adding custom functionality.
//! - **Multi-Provider**: Works with OpenAI, Azure OpenAI, local models, and any OpenAI-compatible API.
//! - **Type-Safe**: Leverages Rust's type system for reliability.
//! - **Async**: Built on Tokio for high-performance async operations.

// Modules

/// Defines the `Agent` struct and its associated builder, which are central to the Helios Engine.
pub mod agent;

/// Provides chat-related functionality, including `ChatMessage`, `ChatSession`, and `Role`.
pub mod chat;

/// Handles configuration for the engine, including LLM providers and agent settings.
pub mod config;

/// Defines the custom `HeliosError` and `Result` types for error handling.
pub mod error;

/// Manages interactions with Large Language Models (LLMs), including different providers.
pub mod llm;

/// Contains the tool system, including the `Tool` trait and various tool implementations.
pub mod tools;

/// Provides HTTP server functionality for exposing OpenAI-compatible API endpoints.
pub mod serve;

// Re-exports

/// Re-export of the `Agent` and `AgentBuilder` for convenient access.
pub use agent::{Agent, AgentBuilder};

/// Re-export of chat-related types.
pub use chat::{ChatMessage, ChatSession, Role};

/// Re-export of configuration types.
pub use config::{Config, LLMConfig, LocalConfig};

/// Re-export of the custom error and result types.
pub use error::{HeliosError, Result};

/// Re-export of LLM-related types.
pub use llm::{
    Delta, LLMClient, LLMProvider, LLMRequest, LLMResponse, LocalLLMProvider, StreamChoice,
    StreamChunk,
};
pub use tools::{
    CalculatorTool, EchoTool, FileEditTool, FileReadTool, FileSearchTool, FileWriteTool,
    MemoryDBTool, QdrantRAGTool, Tool, ToolParameter, ToolRegistry, ToolResult,
};

/// Re-export of serve functionality.
pub use serve::{start_server, start_server_with_agent, ServerState};
