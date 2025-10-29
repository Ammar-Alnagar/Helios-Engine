use thiserror::Error;

#[derive(Error, Debug)]
pub enum HeliosError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("LLM error: {0}")]
    LLMError(String),

    #[error("Tool error: {0}")]
    ToolError(String),

    #[error("Agent error: {0}")]
    AgentError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Llama C++ error: {0}")]
    LlamaCppError(String),
}

pub type Result<T> = std::result::Result<T, HeliosError>;

#[cfg(test)]
mod tests {
    use super::*;

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
