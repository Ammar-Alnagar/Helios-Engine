//! # Configuration Module
//!
//! This module defines the data structures for configuring the Helios Engine.
//! It includes settings for both remote and local Language Models (LLMs),
//! and provides methods for loading and saving configurations from/to TOML files.

use crate::error::{HeliosError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// The main configuration for the Helios Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The configuration for the remote LLM.
    pub llm: LLMConfig,
    /// The configuration for the local LLM (optional).
    #[cfg(feature = "local")]
    #[serde(default)]
    pub local: Option<LocalConfig>,
}

/// Configuration for a remote Language Model (LLM).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// The name of the model to use.
    pub model_name: String,
    /// The base URL of the LLM API.
    pub base_url: String,
    /// The API key for the LLM API.
    pub api_key: String,
    /// The temperature to use for the LLM.
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// The maximum number of tokens to generate.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

/// Configuration for a local Language Model (LLM).
#[cfg(feature = "local")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    /// The Hugging Face repository of the model.
    pub huggingface_repo: String,
    /// The model file to use.
    pub model_file: String,
    /// The context size to use for the LLM.
    #[serde(default = "default_context_size")]
    pub context_size: usize,
    /// The temperature to use for the LLM.
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// The maximum number of tokens to generate.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

/// Returns the default temperature value.
fn default_temperature() -> f32 {
    0.7
}

/// Returns the default maximum number of tokens.
fn default_max_tokens() -> u32 {
    2048
}

/// Returns the default context size.
#[cfg(feature = "local")]
fn default_context_size() -> usize {
    2048
}

impl Config {
    /// Loads the configuration from a TOML file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the TOML file.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded `Config`.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| HeliosError::ConfigError(format!("Failed to read config file: {}", e)))?;

        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Creates a new default configuration.
    pub fn new_default() -> Self {
        Self {
            llm: LLMConfig {
                model_name: "gpt-3.5-turbo".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
                api_key: "your-api-key-here".to_string(),
                temperature: 0.7,
                max_tokens: 2048,
            },
            #[cfg(feature = "local")]
            local: None,
        }
    }

    /// Saves the configuration to a TOML file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the TOML file.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| HeliosError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, content)
            .map_err(|e| HeliosError::ConfigError(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Tests loading a configuration from a file.
    #[test]
    #[cfg(feature = "local")]
    fn test_config_from_file() {
        let config_content = r#"
[llm]
model_name = "gpt-4"
base_url = "https://api.openai.com/v1"
api_key = "test-key"
temperature = 0.7
max_tokens = 2048

[local]
huggingface_repo = "test/repo"
model_file = "model.gguf"
context_size = 4096
temperature = 0.5
max_tokens = 1024
"#;
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, config_content).unwrap();

        let config = Config::from_file(&config_path).unwrap();
        assert_eq!(config.llm.model_name, "gpt-4");
        assert_eq!(config.local.as_ref().unwrap().huggingface_repo, "test/repo");
    }

    /// Tests loading a configuration from a file without local config.
    #[test]
    #[cfg(not(feature = "local"))]
    fn test_config_from_file() {
        let config_content = r#"
[llm]
model_name = "gpt-4"
base_url = "https://api.openai.com/v1"
api_key = "test-key"
temperature = 0.7
max_tokens = 2048
"#;
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, config_content).unwrap();

        let config = Config::from_file(&config_path).unwrap();
        assert_eq!(config.llm.model_name, "gpt-4");
    }

    /// Tests creating a new default configuration.
    #[test]
    fn test_config_new_default() {
        let config = Config::new_default();
        assert_eq!(config.llm.model_name, "gpt-3.5-turbo");
        assert_eq!(config.llm.base_url, "https://api.openai.com/v1");
        assert_eq!(config.llm.api_key, "your-api-key-here");
        assert_eq!(config.llm.temperature, 0.7);
        assert_eq!(config.llm.max_tokens, 2048);
        #[cfg(feature = "local")]
        assert!(config.local.is_none());
    }

    /// Tests saving a configuration to a file.
    #[test]
    fn test_config_save() {
        let config = Config::new_default();
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        config.save(&config_path).unwrap();
        assert!(config_path.exists());

        let loaded_config = Config::from_file(&config_path).unwrap();
        assert_eq!(loaded_config.llm.model_name, config.llm.model_name);
    }

    /// Tests the default value functions.
    #[test]
    fn test_default_functions() {
        assert_eq!(default_temperature(), 0.7);
        assert_eq!(default_max_tokens(), 2048);
        #[cfg(feature = "local")]
        assert_eq!(default_context_size(), 2048);
    }
}
