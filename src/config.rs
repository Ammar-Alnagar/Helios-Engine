use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::error::{HeliosError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub llm: LLMConfig,
    #[serde(default)]
    pub local: Option<LocalConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub model_name: String,
    pub base_url: String,
    pub api_key: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    pub huggingface_repo: String,
    pub model_file: String,
    #[serde(default = "default_context_size")]
    pub context_size: usize,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> u32 {
    2048
}

fn default_context_size() -> usize {
    2048
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| HeliosError::ConfigError(format!("Failed to read config file: {}", e)))?;
        
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Create a default configuration
    pub fn new_default() -> Self {
        Self {
            llm: LLMConfig {
                model_name: "gpt-3.5-turbo".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
                api_key: "your-api-key-here".to_string(),
                temperature: 0.7,
                max_tokens: 2048,
            },
            local: None,
        }
    }

    /// Save configuration to a TOML file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| HeliosError::ConfigError(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(path, content)
            .map_err(|e| HeliosError::ConfigError(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }
}
