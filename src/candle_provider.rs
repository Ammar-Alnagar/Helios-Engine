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
    candle_core::{quantized::gguf_file, Device},
    candle_transformers::generation::LogitsProcessor,
    candle_transformers::models::quantized_qwen2::ModelWeights as Qwen2Model,
    hf_hub::api::sync::Api,
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
    #[cfg(feature = "candle")]
    device: Arc<Device>,
    #[cfg(feature = "candle")]
    tokenizer: Arc<Tokenizer>,
    #[cfg(feature = "candle")]
    model: Arc<std::sync::Mutex<Qwen2Model>>,
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
            let (model_path, tokenizer_path) = Self::download_model_and_tokenizer(&config).await?;

            // Load tokenizer
            let tokenizer = Tokenizer::from_file(&tokenizer_path)
                .map_err(|e| HeliosError::LLMError(format!("Failed to load tokenizer: {}", e)))?;

            // Load model from GGUF
            let mut file = std::fs::File::open(&model_path)
                .map_err(|e| HeliosError::LLMError(format!("Failed to open model file: {}", e)))?;
            let model_content = gguf_file::Content::read(&mut file)
                .map_err(|e| HeliosError::LLMError(format!("Failed to read GGUF file: {}", e)))?;
            let model = Qwen2Model::from_gguf(model_content, &mut file, &device)
                .map_err(|e| HeliosError::LLMError(format!("Failed to load model: {}", e)))?;

            Ok(Self {
                config,
                model_type,
                device: Arc::new(device),
                tokenizer: Arc::new(tokenizer),
                model: Arc::new(std::sync::Mutex::new(model)),
            })
        }
    }

    /// Download model and tokenizer from HuggingFace
    async fn download_model_and_tokenizer(config: &CandleConfig) -> Result<(PathBuf, PathBuf)> {
        #[cfg(feature = "candle")]
        {
            // First, try to find model in local cache
            if let Some((cached_model_path, cached_tokenizer_path)) =
                Self::find_model_in_cache(&config.huggingface_repo, &config.model_file)
            {
                return Ok((cached_model_path, cached_tokenizer_path));
            }

            // If not in cache, download from HuggingFace
            let api = Api::new().map_err(|e| {
                HeliosError::LLMError(format!("Failed to initialize HF API: {}", e))
            })?;

            // Download model file
            let repo_api = api.model(config.huggingface_repo.clone());
            let model_path = repo_api.get(&config.model_file).map_err(|e| {
                HeliosError::LLMError(format!(
                    "Failed to download model file {}: {}",
                    config.model_file, e
                ))
            })?;

            // For GGUF models, try to get tokenizer from compatible base repos
            let base_repos = vec![
                "Qwen/Qwen2.5-0.5B-Instruct", // Qwen2.5
                "Qwen/Qwen2-0.5B-Instruct",   // Qwen2
            ];

            let tokenizer_path = base_repos.iter()
                .find_map(|repo| Self::find_tokenizer_in_cache(repo))
                .or_else(|| {
                    // Try to download from the first base repo
                    let tok_api = Api::new().ok()?;
                    let tok_repo = tok_api.model(base_repos[0].to_string());
                    tok_repo.get("tokenizer.json").ok()
                })
                .ok_or_else(|| HeliosError::LLMError("Failed to find or download tokenizer.json".to_string()))?;

            Ok((model_path, tokenizer_path))
        }

        #[cfg(not(feature = "candle"))]
        {
            Err(HeliosError::LLMError(
                "Candle feature is not enabled".to_string(),
            ))
        }
    }

    /// Find tokenizer in local HuggingFace cache
    fn find_tokenizer_in_cache(repo: &str) -> Option<PathBuf> {
        // Get HuggingFace cache directory
        let cache_dir = std::env::var("HF_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".cache").join("huggingface")
            });

        let hub_dir = cache_dir.join("hub");

        // Convert repo name to HuggingFace cache format
        let cache_repo_name = format!("models--{}", repo.replace("/", "--"));
        let repo_dir = hub_dir.join(&cache_repo_name);

        if !repo_dir.exists() {
            return None;
        }

        // Check in snapshots directory
        let snapshots_dir = repo_dir.join("snapshots");
        if snapshots_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
                for entry in entries.flatten() {
                    let snapshot_path = entry.path();
                    let tokenizer_path = snapshot_path.join("tokenizer.json");
                    if tokenizer_path.exists() {
                        return Some(tokenizer_path);
                    }
                }
            }
        }

        None
    }

    /// Find model and tokenizer in local HuggingFace cache
    fn find_model_in_cache(repo: &str, model_file: &str) -> Option<(PathBuf, PathBuf)> {
        // Get HuggingFace cache directory
        let cache_dir = std::env::var("HF_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".cache").join("huggingface")
            });

        let hub_dir = cache_dir.join("hub");

        // Convert repo name to HuggingFace cache format
        // e.g., "unsloth/Qwen3-0.6B-GGUF" -> "models--unsloth--Qwen3-0.6B-GGUF"
        let cache_repo_name = format!("models--{}", repo.replace("/", "--"));
        let repo_dir = hub_dir.join(&cache_repo_name);

        if !repo_dir.exists() {
            return None;
        }

        // Check in snapshots directory (standard HuggingFace cache format)
        let snapshots_dir = repo_dir.join("snapshots");
        if snapshots_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
                for entry in entries.flatten() {
                    let snapshot_path = entry.path();

                    // Look for model file
                    let model_path = snapshot_path.join(model_file);
                    if model_path.exists() {
                        // For GGUF repos, tokenizer is in base repo
                        let base_repo = if repo.contains("-GGUF") {
                            repo.trim_end_matches("-GGUF")
                        } else {
                            repo
                        };
                        if let Some(tokenizer_path) = Self::find_tokenizer_in_cache(base_repo) {
                            return Some((model_path, tokenizer_path));
                        }
                    }
                }
            }
        }

        None
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

    /// Run inference on the model
    pub async fn inference(&self, prompt: &str, max_tokens: u32) -> Result<String> {
        #[cfg(feature = "candle")]
        {
            // Implement inference for Qwen models
            match self.model_type {
                ModelType::Qwen | ModelType::Qwen2 | ModelType::Qwen3 => {
                    self.inference_qwen(prompt, max_tokens).await
                }
                _ => Err(HeliosError::LLMError(format!(
                    "Inference not yet implemented for {:?} models",
                    self.model_type
                ))),
            }
        }

        #[cfg(not(feature = "candle"))]
        {
            Err(HeliosError::LLMError(
                "Candle feature is not enabled".to_string(),
            ))
        }
    }

    /// Inference for Qwen models
    #[cfg(feature = "candle")]
    async fn inference_qwen(&self, prompt: &str, max_tokens: u32) -> Result<String> {
        // Tokenize the prompt
        let tokens = self.tokenizer
            .encode(prompt, true)
            .map_err(|e| HeliosError::LLMError(format!("Tokenization error: {}", e)))?
            .get_ids()
            .to_vec();

        if tokens.is_empty() {
            return Err(HeliosError::LLMError("Empty token sequence".to_string()));
        }

        // Run inference in a blocking operation
        let device = self.device.clone();
        let tokenizer = self.tokenizer.clone();
        let model = self.model.clone();
            let max_tokens = max_tokens as usize;

        let result = tokio::task::block_in_place(move || {
            let mut model = model.lock().map_err(|e| HeliosError::LLMError(format!("Model lock error: {}", e)))?;

            // Create logits processor
            let mut logits_processor = LogitsProcessor::new(299792458, None, None);

            // Generate tokens
            let mut generated_tokens = tokens.clone();
            let mut next_token = *generated_tokens.last().unwrap();

            for index in 0..max_tokens {
                // Create input tensor with just the next token (autoregressive)
                let input = candle_core::Tensor::new(&[next_token], &*device)?
                    .unsqueeze(0)?;

                // Forward pass - position is the current index
                let logits = model.forward(&input, index)?;
                let logits = logits.squeeze(0)?;

                // Sample next token
                next_token = logits_processor.sample(&logits)?;

                generated_tokens.push(next_token);

                // Check for end tokens
                if let Some(im_end) = tokenizer.token_to_id("<|im_end|>") {
                    if next_token == im_end {
                        break;
                    }
                }
                if let Some(eot) = tokenizer.token_to_id("<|endoftext|>") {
                    if next_token == eot {
                        break;
                    }
                }
                // Also break on newline or common stop sequences
                if next_token == tokenizer.token_to_id("\n").unwrap_or(0) {
                    break;
                }
            }

            // Decode the generated tokens (skip prompt tokens)
            let output_tokens = &generated_tokens[tokens.len()..];
            let output = tokenizer
                .decode(output_tokens, true)
                .map_err(|e| HeliosError::LLMError(format!("Decode error: {}", e)))?;

            Ok(output)
        });

        result
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
