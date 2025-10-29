use crate::chat::ChatMessage;
use crate::config::{LLMConfig, LocalConfig};
use crate::error::{HeliosError, Result};
use crate::tools::ToolDefinition;
use async_trait::async_trait;
use futures::stream::StreamExt;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use llama_cpp_2::token::LlamaToken;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::task;

// Add From trait for LLamaCppError to convert to HeliosError
impl From<llama_cpp_2::LLamaCppError> for HeliosError {
    fn from(err: llama_cpp_2::LLamaCppError) -> Self {
        HeliosError::LlamaCppError(format!("{:?}", err))
    }
}

#[derive(Clone)]
pub enum LLMProviderType {
    Remote(LLMConfig),
    Local(LocalConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChoice {
    pub index: u32,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse>;
}

pub struct LLMClient {
    provider: Box<dyn LLMProvider + Send + Sync>,
    provider_type: LLMProviderType,
}

impl LLMClient {
    pub async fn new(provider_type: LLMProviderType) -> Result<Self> {
        let provider: Box<dyn LLMProvider + Send + Sync> = match &provider_type {
            LLMProviderType::Remote(config) => Box::new(RemoteLLMClient::new(config.clone())),
            LLMProviderType::Local(config) => {
                Box::new(LocalLLMProvider::new(config.clone()).await?)
            }
        };

        Ok(Self {
            provider,
            provider_type,
        })
    }

    pub fn provider_type(&self) -> &LLMProviderType {
        &self.provider_type
    }
}

// Rename the old LLMClient to RemoteLLMClient
pub struct RemoteLLMClient {
    config: LLMConfig,
    client: Client,
}

impl RemoteLLMClient {
    pub fn new(config: LLMConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub fn config(&self) -> &LLMConfig {
        &self.config
    }
}

pub struct LocalLLMProvider {
    model: Arc<LlamaModel>,
}

impl LocalLLMProvider {
    pub async fn new(config: LocalConfig) -> Result<Self> {
        // Initialize llama backend
        let backend = LlamaBackend::init().map_err(|e| {
            HeliosError::LLMError(format!("Failed to initialize llama backend: {:?}", e))
        })?;

        // Download model from HuggingFace if needed
        let model_path = Self::download_model(&config).await?;

        // Load the model
        let model_params = LlamaModelParams::default().with_n_gpu_layers(99); // Use GPU if available

        let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)
            .map_err(|e| HeliosError::LLMError(format!("Failed to load model: {:?}", e)))?;

        Ok(Self {
            model: Arc::new(model),
        })
    }

    async fn download_model(config: &LocalConfig) -> Result<std::path::PathBuf> {
        use std::process::Command;

        // Use huggingface_hub to download the model
        let output = Command::new("huggingface-cli")
            .args(&[
                "download",
                &config.huggingface_repo,
                &config.model_file,
                "--local-dir",
                ".cache/models",
                "--local-dir-use-symlinks",
                "False",
            ])
            .output()
            .map_err(|e| HeliosError::LLMError(format!("Failed to run huggingface-cli: {}", e)))?;

        if !output.status.success() {
            return Err(HeliosError::LLMError(format!(
                "Failed to download model: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let model_path = std::path::PathBuf::from(".cache/models").join(&config.model_file);
        if !model_path.exists() {
            return Err(HeliosError::LLMError(format!(
                "Model file not found after download: {}",
                model_path.display()
            )));
        }

        Ok(model_path)
    }

    fn format_messages(&self, messages: &[ChatMessage]) -> String {
        let mut formatted = String::new();
        for message in messages {
            match message.role {
                crate::chat::Role::System => {
                    formatted.push_str(&format!("<|im_start|>system\n{}\n<|im_end|>\n", message.content));
                }
                crate::chat::Role::User => {
                    formatted.push_str(&format!("<|im_start|>user\n{}\n<|im_end|>\n", message.content));
                }
                crate::chat::Role::Assistant => {
                    formatted.push_str(&format!("<|im_start|>assistant\n{}\n<|im_end|>\n", message.content));
                }
                crate::chat::Role::Tool => {
                    formatted.push_str(&format!("Tool result: {}\n", message.content));
                }
            }
        }
        formatted.push_str("<|im_start|>assistant\n");
        formatted
    }
}

#[async_trait]
impl LLMProvider for RemoteLLMClient {
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse> {
        let url = format!("{}/chat/completions", self.config.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::LLMError(format!(
                "LLM API request failed with status {}: {}",
                status, error_text
            )));
        }

        let llm_response: LLMResponse = response.json().await?;
        Ok(llm_response)
    }
}

impl RemoteLLMClient {
    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<ChatMessage> {
        let request = LLMRequest {
            model: self.config.model_name.clone(),
            messages,
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens),
            tools,
            tool_choice: None,
            stream: None,
        };

        let response = self.generate(request).await?;

        response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message)
            .ok_or_else(|| HeliosError::LLMError("No response from LLM".to_string()))
    }

    pub async fn chat_stream<F>(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
        mut on_chunk: F,
    ) -> Result<ChatMessage>
    where
        F: FnMut(&str) + Send,
    {
        let request = LLMRequest {
            model: self.config.model_name.clone(),
            messages,
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens),
            tools,
            tool_choice: None,
            stream: Some(true),
        };

        let url = format!("{}/chat/completions", self.config.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::LLMError(format!(
                "LLM API request failed with status {}: {}",
                status, error_text
            )));
        }

        let mut stream = response.bytes_stream();
        let mut full_content = String::new();
        let mut role = None;
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);

            // Process complete lines
            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() || line == "data: [DONE]" {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    match serde_json::from_str::<StreamChunk>(data) {
                        Ok(stream_chunk) => {
                            if let Some(choice) = stream_chunk.choices.first() {
                                if let Some(r) = &choice.delta.role {
                                    role = Some(r.clone());
                                }
                                if let Some(content) = &choice.delta.content {
                                    full_content.push_str(content);
                                    on_chunk(content);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::debug!("Failed to parse stream chunk: {} - Data: {}", e, data);
                        }
                    }
                }
            }
        }

        Ok(ChatMessage {
            role: crate::chat::Role::from(role.as_deref().unwrap_or("assistant")),
            content: full_content,
            name: None,
            tool_calls: None,
            tool_call_id: None,
        })
    }
}

#[async_trait]
impl LLMProvider for LocalLLMProvider {
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse> {
        let prompt = self.format_messages(&request.messages);
        let model = Arc::clone(&self.model);

        // Run inference in a blocking task
        let result = task::spawn_blocking(move || {
            // Initialize backend
            let backend = LlamaBackend::init().map_err(|e| {
                HeliosError::LLMError(format!("Failed to initialize backend: {:?}", e))
            })?;

            // Create context
            use std::num::NonZeroU32;
            let ctx_params =
                LlamaContextParams::default().with_n_ctx(Some(NonZeroU32::new(2048).unwrap()));

            let mut context = model
                .new_context(&backend, ctx_params)
                .map_err(|e| HeliosError::LLMError(format!("Failed to create context: {:?}", e)))?;

            // Tokenize the prompt
            let tokens = context
                .model
                .str_to_token(&prompt, AddBos::Always)
                .map_err(|e| HeliosError::LLMError(format!("Tokenization failed: {:?}", e)))?;

            // Create batch for prompt
            let mut prompt_batch = LlamaBatch::new(tokens.len(), 1);
            for (i, &token) in tokens.iter().enumerate() {
                let compute_logits = true; // Compute logits for all tokens (they accumulate)
                prompt_batch
                    .add(token, i as i32, &[0], compute_logits)
                    .map_err(|e| {
                        HeliosError::LLMError(format!(
                            "Failed to add prompt token to batch: {:?}",
                            e
                        ))
                    })?;
            }

            // Decode the prompt
            context
                .decode(&mut prompt_batch)
                .map_err(|e| HeliosError::LLMError(format!("Failed to decode prompt: {:?}", e)))?;

            // Generate response tokens
            let mut generated_text = String::new();
            let max_new_tokens = 128; // Limit response length
            let mut next_pos = tokens.len() as i32; // Start after the prompt tokens

            for _ in 0..max_new_tokens {
                // Get logits from the last decoded position (get_logits returns logits for the last token)
                let logits = context.get_logits();

                let token_idx = logits
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(idx, _)| idx)
                    .unwrap_or_else(|| {
                        let eos = context.model.token_eos();
                        eos.0 as usize
                    });
                let token = LlamaToken(token_idx as i32);

                // Check for end of sequence
                if token == context.model.token_eos() {
                    break;
                }

                // Convert token back to text
                match context.model.token_to_str(token, Special::Tokenize) {
                    Ok(text) => generated_text.push_str(&text),
                    Err(_) => continue, // Skip invalid tokens
                }

                // Create a new batch with just this token
                let mut gen_batch = LlamaBatch::new(1, 1);
                gen_batch.add(token, next_pos, &[0], true).map_err(|e| {
                    HeliosError::LLMError(format!(
                        "Failed to add generated token to batch: {:?}",
                        e
                    ))
                })?;

                // Decode the new token
                context.decode(&mut gen_batch).map_err(|e| {
                    HeliosError::LLMError(format!("Failed to decode token: {:?}", e))
                })?;

                next_pos += 1;
            }

            Ok::<String, HeliosError>(generated_text)
        })
        .await
        .map_err(|e| HeliosError::LLMError(format!("Task failed: {}", e)))??;

        let response = LLMResponse {
            id: format!("local-{}", chrono::Utc::now().timestamp()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: "local-model".to_string(),
            choices: vec![Choice {
                index: 0,
                message: ChatMessage {
                    role: crate::chat::Role::Assistant,
                    content: result,
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Usage {
                prompt_tokens: 0,     // TODO: Calculate actual token count
                completion_tokens: 0, // TODO: Calculate actual token count
                total_tokens: 0,      // TODO: Calculate actual token count
            },
        };

        Ok(response)
    }
}

#[async_trait]
impl LLMProvider for LLMClient {
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse> {
        self.provider.generate(request).await
    }
}

impl LLMClient {
    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<ChatMessage> {
        let (model_name, temperature, max_tokens) = match &self.provider_type {
            LLMProviderType::Remote(config) => (
                config.model_name.clone(),
                config.temperature,
                config.max_tokens,
            ),
            LLMProviderType::Local(config) => (
                "local-model".to_string(),
                config.temperature,
                config.max_tokens,
            ),
        };

        let request = LLMRequest {
            model: model_name,
            messages,
            temperature: Some(temperature),
            max_tokens: Some(max_tokens),
            tools,
            tool_choice: None,
            stream: None,
        };

        let response = self.generate(request).await?;

        response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message)
            .ok_or_else(|| HeliosError::LLMError("No response from LLM".to_string()))
    }

    pub async fn chat_stream<F>(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
        on_chunk: F,
    ) -> Result<ChatMessage>
    where
        F: FnMut(&str) + Send,
    {
        // For local models, streaming is not yet implemented, so fall back to regular chat
        match &self.provider_type {
            LLMProviderType::Remote(config) => {
                let remote_client = RemoteLLMClient::new(config.clone());
                remote_client.chat_stream(messages, tools, on_chunk).await
            }
            LLMProviderType::Local(_) => {
                // For now, local models don't support streaming
                // TODO: Implement streaming for local models
                self.chat(messages, tools).await
            }
        }
    }
}
