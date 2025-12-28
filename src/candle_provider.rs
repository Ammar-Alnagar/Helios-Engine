//! # Candle Provider Module
//!
//! This module provides support for running local language models using the Candle backend.
//! It automatically detects the model type and architecture, and uses the appropriate
//! inference code from the candle-transformers library.

use crate::chat::ChatMessage;
use crate::config::CandleConfig;
use crate::error::{HeliosError, Result};
use crate::llm::{Choice, LLMProvider, LLMRequest, LLMResponse, Usage};
use async_trait::async_trait;
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

#[cfg(feature = "candle")]
use {
    candle_core::Device,
    hf_hub::{api::sync::Api, Repo, RepoType},
    tokenizers::Tokenizer,
};

/// Model type enumeration for supported architectures
#[derive(Debug, Clone, PartialEq)]
pub enum ModelType {
    Qwen,
    Qwen2,
    Qwen3,
    Llama,
    Llama2,
    Gemma,
    Gemma2,
    Mistral,
    Other(String),
}

impl ModelType {
    /// Detect model type from repository name
    pub fn from_repo(repo: &str) -> Self {
        let repo_lower = repo.to_lowercase();
        if repo_lower.contains("qwen3") {
            ModelType::Qwen3
        } else if repo_lower.contains("qwen2") {
            ModelType::Qwen2
        } else if repo_lower.contains("qwen") {
            ModelType::Qwen
        } else if repo_lower.contains("llama2") {
            ModelType::Llama2
        } else if repo_lower.contains("llama") {
            ModelType::Llama
        } else if repo_lower.contains("gemma2") {
            ModelType::Gemma2
        } else if repo_lower.contains("gemma") {
            ModelType::Gemma
        } else if repo_lower.contains("mistral") {
            ModelType::Mistral
        } else {
            ModelType::Other(repo.to_string())
        }
    }
}

/// A token output stream for handling model token generation
#[cfg(feature = "candle")]
pub struct TokenOutputStream {
    tokenizer: tokenizers::Tokenizer,
    tokens: Vec<u32>,
    prev_index: usize,
    current_index: usize,
}

#[cfg(feature = "candle")]
impl TokenOutputStream {
    pub fn new(tokenizer: Tokenizer) -> Self {
        Self {
            tokenizer,
            tokens: Vec::new(),
            prev_index: 0,
            current_index: 0,
        }
    }

    pub fn next_token(&mut self, token: u32) -> Result<Option<String>> {
        self.tokens.push(token);
        self.current_index += 1;

        let text = self
            .tokenizer
            .decode(&[token], true)
            .map_err(|e| HeliosError::LLMError(format!("Tokenizer error: {}", e)))?;

        if !text.is_empty() {
            return Ok(Some(text));
        }
        Ok(None)
    }

    pub fn decode_all(&self) -> Result<String> {
        self.tokenizer
            .decode(&self.tokens, true)
            .map_err(|e| HeliosError::LLMError(format!("Tokenizer decode error: {}", e)))
    }

    pub fn clear(&mut self) {
        self.tokens.clear();
        self.prev_index = 0;
        self.current_index = 0;
    }

    pub fn tokenizer(&self) -> &tokenizers::Tokenizer {
        &self.tokenizer
    }

    pub fn get_token(&self, token_str: &str) -> Option<u32> {
        self.tokenizer.token_to_id(token_str)
    }
}

/// Candle LLM Provider for running models locally
pub struct CandleLLMProvider {
    config: CandleConfig,
    model_type: ModelType,
    #[allow(dead_code)]
    device: Arc<Device>,
    #[allow(dead_code)]
    tokenizer: Arc<Tokenizer>,
}

impl CandleLLMProvider {
    /// Creates a new Candle LLM provider
    pub async fn new(config: CandleConfig) -> Result<Self> {
        #[cfg(not(feature = "candle"))]
        {
            return Err(HeliosError::LLMError(
                "Candle feature is not enabled. Please enable the 'candle' feature in Cargo.toml"
                    .to_string(),
            ));
        }

        #[cfg(feature = "candle")]
        {
            let model_type = ModelType::from_repo(&config.huggingface_repo);

            // Determine device
            let device = if config.use_gpu {
                match Device::cuda_if_available(0) {
                    Ok(device) => device,
                    Err(_) => Device::Cpu,
                }
            } else {
                Device::Cpu
            };

            // Download model and tokenizer
            let (_model_path, tokenizer_path) = Self::download_model_and_tokenizer(&config).await?;

            // Load tokenizer
            let tokenizer = Tokenizer::from_file(&tokenizer_path)
                .map_err(|e| HeliosError::LLMError(format!("Failed to load tokenizer: {}", e)))?;

            Ok(Self {
                config,
                model_type,
                device: Arc::new(device),
                tokenizer: Arc::new(tokenizer),
            })
        }
    }

    /// Download model and tokenizer from HuggingFace
    async fn download_model_and_tokenizer(config: &CandleConfig) -> Result<(PathBuf, PathBuf)> {
        #[cfg(feature = "candle")]
        {
            let api = Api::new().map_err(|e| {
                HeliosError::LLMError(format!("Failed to initialize HF API: {}", e))
            })?;
            let repo = api.repo(Repo::new(config.huggingface_repo.clone(), RepoType::Model));

            // Download model file
            let model_path = repo.get(&config.model_file).map_err(|e| {
                HeliosError::LLMError(format!(
                    "Failed to download model file {}: {}",
                    config.model_file, e
                ))
            })?;

            // Download tokenizer (try common names)
            let tokenizer_names = vec![
                "tokenizer.json",
                "tokenizer.model",
                "special_tokens_map.json",
            ];

            let mut tokenizer_path = None;
            for name in &tokenizer_names {
                if let Ok(path) = repo.get(name) {
                    tokenizer_path = Some(path);
                    break;
                }
            }

            let tokenizer_path = tokenizer_path.ok_or_else(|| {
                HeliosError::LLMError("Failed to find tokenizer file".to_string())
            })?;

            Ok((model_path, tokenizer_path))
        }

        #[cfg(not(feature = "candle"))]
        {
            Err(HeliosError::LLMError(
                "Candle feature is not enabled".to_string(),
            ))
        }
    }

    /// Format messages into a prompt string
    fn format_messages(&self, messages: &[ChatMessage]) -> String {
        match self.model_type {
            ModelType::Qwen | ModelType::Qwen2 | ModelType::Qwen3 => {
                self.format_qwen_messages(messages)
            }
            ModelType::Llama | ModelType::Llama2 => self.format_llama_messages(messages),
            ModelType::Gemma | ModelType::Gemma2 => self.format_gemma_messages(messages),
            ModelType::Mistral => self.format_mistral_messages(messages),
            ModelType::Other(_) => self.format_default_messages(messages),
        }
    }

    /// Format messages for Qwen models
    fn format_qwen_messages(&self, messages: &[ChatMessage]) -> String {
        let mut formatted = String::new();
        for message in messages {
            match message.role {
                crate::chat::Role::System => {
                    formatted.push_str("<|im_start|>system\n");
                    formatted.push_str(&message.content);
                    formatted.push_str("\n<|im_end|>\n");
                }
                crate::chat::Role::User => {
                    formatted.push_str("<|im_start|>user\n");
                    formatted.push_str(&message.content);
                    formatted.push_str("\n<|im_end|>\n");
                }
                crate::chat::Role::Assistant => {
                    formatted.push_str("<|im_start|>assistant\n");
                    formatted.push_str(&message.content);
                    formatted.push_str("\n<|im_end|>\n");
                }
                crate::chat::Role::Tool => {
                    formatted.push_str("<|im_start|>tool\n");
                    formatted.push_str(&message.content);
                    formatted.push_str("\n<|im_end|>\n");
                }
            }
        }
        formatted.push_str("<|im_start|>assistant\n");
        formatted
    }

    /// Format messages for Llama models
    fn format_llama_messages(&self, messages: &[ChatMessage]) -> String {
        let mut formatted = String::new();
        for message in messages {
            match message.role {
                crate::chat::Role::System => {
                    formatted.push_str("[INST] <<SYS>>\n");
                    formatted.push_str(&message.content);
                    formatted.push_str("\n<</SYS>>\n\n");
                }
                crate::chat::Role::User => {
                    if !formatted.is_empty() && !formatted.ends_with("[INST] ") {
                        formatted.push_str("[INST] ");
                    }
                    formatted.push_str(&message.content);
                    formatted.push_str(" [/INST] ");
                }
                crate::chat::Role::Assistant => {
                    formatted.push_str(&message.content);
                    formatted.push_str(" </s><s>[INST] ");
                }
                crate::chat::Role::Tool => {
                    formatted.push_str(&message.content);
                    formatted.push_str(" </s><s>[INST] ");
                }
            }
        }
        formatted
    }

    /// Format messages for Gemma models
    fn format_gemma_messages(&self, messages: &[ChatMessage]) -> String {
        let mut formatted = String::new();
        for message in messages {
            match message.role {
                crate::chat::Role::System => {
                    formatted.push_str(&message.content);
                }
                crate::chat::Role::User => {
                    formatted.push_str("<start_of_turn>user\n");
                    formatted.push_str(&message.content);
                    formatted.push_str("<end_of_turn>\n");
                }
                crate::chat::Role::Assistant => {
                    formatted.push_str("<start_of_turn>model\n");
                    formatted.push_str(&message.content);
                    formatted.push_str("<end_of_turn>\n");
                }
                crate::chat::Role::Tool => {
                    formatted.push_str(&message.content);
                }
            }
        }
        formatted.push_str("<start_of_turn>model\n");
        formatted
    }

    /// Format messages for Mistral models
    fn format_mistral_messages(&self, messages: &[ChatMessage]) -> String {
        let mut formatted = String::new();
        for message in messages {
            match message.role {
                crate::chat::Role::System => {
                    formatted.push_str(&message.content);
                    formatted.push_str("\n\n");
                }
                crate::chat::Role::User => {
                    formatted.push_str("[INST] ");
                    formatted.push_str(&message.content);
                    formatted.push_str(" [/INST]");
                }
                crate::chat::Role::Assistant => {
                    formatted.push_str(&message.content);
                    formatted.push_str("</s>[INST] ");
                }
                crate::chat::Role::Tool => {
                    formatted.push_str(&message.content);
                    formatted.push_str("</s>[INST] ");
                }
            }
        }
        formatted
    }

    /// Format messages for unknown models
    fn format_default_messages(&self, messages: &[ChatMessage]) -> String {
        let mut formatted = String::new();
        for message in messages {
            let role_str = match message.role {
                crate::chat::Role::System => "SYSTEM",
                crate::chat::Role::User => "USER",
                crate::chat::Role::Assistant => "ASSISTANT",
                crate::chat::Role::Tool => "TOOL",
            };
            formatted.push_str(&format!("{}: {}\n", role_str, message.content));
        }
        formatted
    }

    /// Run inference on the model (stub for now - will be implemented with actual model loading)
    pub async fn inference(&self, _prompt: &str, _max_tokens: u32) -> Result<String> {
        // This will be implemented when we add the actual model loading logic
        Err(HeliosError::LLMError(
            "Inference not yet implemented for Candle provider. Model architecture detection and loading is in progress."
                .to_string(),
        ))
    }
}

#[async_trait]
impl LLMProvider for CandleLLMProvider {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse> {
        // Format the messages into a prompt
        let prompt = self.format_messages(&request.messages);

        // Get max tokens from request or use config default
        let max_tokens = request.max_tokens.unwrap_or(self.config.max_tokens);

        // Run inference
        let content = self.inference(&prompt, max_tokens).await?;

        // Create response
        let response = LLMResponse {
            id: format!("candle-{}", Uuid::new_v4()),
            object: "text_completion".to_string(),
            created: Utc::now().timestamp() as u64,
            model: self.config.huggingface_repo.clone(),
            choices: vec![Choice {
                index: 0,
                message: ChatMessage::assistant(content),
                finish_reason: Some("stop".to_string()),
            }],
            usage: Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
        };

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_type_detection() {
        assert_eq!(ModelType::from_repo("unsloth/Qwen3-7B"), ModelType::Qwen3);
        assert_eq!(ModelType::from_repo("unsloth/Qwen2-7B"), ModelType::Qwen2);
        assert_eq!(ModelType::from_repo("unsloth/Qwen-7B"), ModelType::Qwen);
        assert_eq!(
            ModelType::from_repo("meta-llama/Llama-2-7b"),
            ModelType::Llama2
        );
        assert_eq!(
            ModelType::from_repo("meta-llama/Llama-7b"),
            ModelType::Llama
        );
        assert_eq!(ModelType::from_repo("google/gemma-7b"), ModelType::Gemma);
        assert_eq!(
            ModelType::from_repo("mistralai/Mistral-7B"),
            ModelType::Mistral
        );
    }

    #[test]
    fn test_format_qwen_messages() {
        let provider = CandleLLMProvider {
            config: CandleConfig {
                huggingface_repo: "test/qwen".to_string(),
                model_file: "model.safetensors".to_string(),
                context_size: 2048,
                temperature: 0.7,
                max_tokens: 1024,
                use_gpu: false,
            },
            model_type: ModelType::Qwen,
            device: Arc::new(Device::Cpu),
            tokenizer: Arc::new(Tokenizer::new()),
        };

        let messages = vec![
            ChatMessage::system("You are a helpful assistant."),
            ChatMessage::user("Hello"),
        ];

        let formatted = provider.format_qwen_messages(&messages);
        assert!(formatted.contains("<|im_start|>system"));
        assert!(formatted.contains("<|im_start|>user"));
        assert!(formatted.contains("<|im_start|>assistant"));
    }
}
