//! # Helios - LLM Agent Framework
//!
//! Helios is a powerful and flexible Rust framework for building LLM-powered agents
//! with tool support, chat capabilities, and easy configuration management.
//!
//! ## Quick Start
//!
//! ### Using as a Library (Direct LLM Calls)
//!
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
//! ### Using with Agent System
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
//! - **Direct LLM Access**: Use `LLMClient` for simple, direct calls to LLM models
//! - **Agent System**: Create intelligent agents with tools and persistent conversation
//! - **Tool Support**: Extensible tool system for adding custom functionality
//! - **Multi-Provider**: Works with OpenAI, Azure OpenAI, local models, and any OpenAI-compatible API
//! - **Type-Safe**: Leverages Rust's type system for reliability
//! - **Async**: Built on Tokio for high-performance async operations

pub mod agent;
pub mod chat;
pub mod config;
pub mod error;
pub mod llm;
pub mod tools;

// Re-export core types for convenient access
pub use agent::{Agent, AgentBuilder};
pub use chat::{ChatMessage, ChatSession, Role};
pub use config::{Config, LLMConfig, LocalConfig};
pub use error::{HeliosError, Result};
pub use llm::{
    Delta, LLMClient, LLMProvider, LLMRequest, LLMResponse, LocalLLMProvider, StreamChoice,
    StreamChunk,
};
pub use tools::{CalculatorTool, EchoTool, FileSearchTool, FileReadTool, FileWriteTool, FileEditTool, Tool, ToolParameter, ToolRegistry, ToolResult};
