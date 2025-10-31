//! # Error and Result Module
//!
//! This module defines the custom error type `HeliosError` for the Helios Engine,
//! and a convenient `Result` type alias.

use thiserror::Error;

/// The custom error type for the Helios Engine.
#[derive(Error, Debug)]
pub enum HeliosError {
    /// An error related to configuration.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// An error related to the Language Model (LLM).
    #[error("LLM error: {0}")]
    LLMError(String),

    /// An error related to a tool.
    #[error("Tool error: {0}")]
    ToolError(String),

    /// An error related to an agent.
    #[error("Agent error: {0}")]
    AgentError(String),

    /// An error related to a network request.
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    /// An error related to serialization or deserialization.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// An I/O error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// An error related to parsing TOML.
    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    /// An error from the Llama C++ backend.
    #[error("Llama C++ error: {0}")]
    LlamaCppError(String),
}

/// A convenient `Result` type alias for the Helios Engine.
pub type Result<T> = std::result::Result<T, HeliosError>;

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that the error types can be created.
    #[test]
    fn test_error_types() {
        let config_error = HeliosError::ConfigError("Config issue".to_string());
        assert!(matches!(config_error, HeliosError::ConfigError(_)));

        let llm_error = HeliosError::LLMError("LLM issue".to_string());
        assert!(matches!(llm_error, HeliosError::LLMError(_)));

        let tool_error = HeliosError::ToolError("Tool issue".to_string());
        assert!(matches!(tool_error, HeliosError::ToolError(_)));

        let agent_error = HeliosError::AgentError("Agent issue".to_string());
        assert!(matches!(agent_error, HeliosError::AgentError(_)));
    }

    /// Tests the display formatting of the error types.
    #[test]
    fn test_error_display() {
        let config_error = HeliosError::ConfigError("Config issue".to_string());
        assert_eq!(
            format!("{}", config_error),
            "Configuration error: Config issue"
        );

        let llm_error = HeliosError::LLMError("LLM issue".to_string());
        assert_eq!(format!("{}", llm_error), "LLM error: LLM issue");

        let tool_error = HeliosError::ToolError("Tool issue".to_string());
        assert_eq!(format!("{}", tool_error), "Tool error: Tool issue");

        let agent_error = HeliosError::AgentError("Agent issue".to_string());
        assert_eq!(format!("{}", agent_error), "Agent error: Agent issue");
    }
}
