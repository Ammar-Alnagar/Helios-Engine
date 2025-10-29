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
