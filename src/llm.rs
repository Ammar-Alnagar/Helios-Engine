//! # LLM Module
//!
//! This module provides the functionality for interacting with Large Language Models (LLMs).
//! It supports both remote LLMs (like OpenAI) and local LLMs (via `llama.cpp`).
//! The `LLMClient` provides a unified interface for both types of providers.

use crate::chat::ChatMessage;
use crate::config::LLMConfig;
use crate::error::{HeliosError, Result};
use crate::tools::ToolDefinition;
use async_trait::async_trait;
use futures::stream::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[cfg(feature = "local")]
use {
    crate::config::LocalConfig,
    llama_cpp_2::{
        context::params::LlamaContextParams,
        llama_backend::LlamaBackend,
        llama_batch::LlamaBatch,
        model::{params::LlamaModelParams, AddBos, LlamaModel, Special},
        token::LlamaToken,
    },
    std::{fs::File, os::fd::AsRawFd, sync::Arc},
    tokio::task,
};

#[cfg(feature = "candle")]
use crate::candle_provider::CandleLLMProvider;

// Add From trait for LlamaCppError to convert to HeliosError
#[cfg(feature = "local")]
impl From<llama_cpp_2::LlamaCppError> for HeliosError {
    fn from(err: llama_cpp_2::LlamaCppError) -> Self {
        HeliosError::LlamaCppError(format!("{:?}", err))
    }
}

/// The type of LLM provider to use.
#[derive(Clone)]
pub enum LLMProviderType {
    /// A remote LLM provider, such as OpenAI.
    Remote(LLMConfig),
    /// A local LLM provider, using `llama.cpp`.
    #[cfg(feature = "local")]
    Local(LocalConfig),
    /// A local LLM provider, using Candle.
    #[cfg(feature = "candle")]
    Candle(crate::config::CandleConfig),
}

/// A request to an LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    /// The model to use for the request.
    pub model: String,
    /// The messages to send to the model.
    pub messages: Vec<ChatMessage>,
    /// The temperature to use for the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// The maximum number of tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// The tools to make available to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    /// The tool choice to use for the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    /// Whether to stream the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Stop sequences for the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

/// A chunk of a streamed response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// The ID of the chunk.
    pub id: String,
    /// The object type.
    pub object: String,
    /// The creation timestamp.
    pub created: u64,
    /// The model that generated the chunk.
    pub model: String,
    /// The choices in the chunk.
    pub choices: Vec<StreamChoice>,
}

/// A choice in a streamed response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChoice {
    /// The index of the choice.
    pub index: u32,
    /// The delta of the choice.
    pub delta: Delta,
    /// The reason the stream finished.
    pub finish_reason: Option<String>,
}

/// A tool call in a streamed delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaToolCall {
    /// The index of the tool call.
    pub index: u32,
    /// The ID of the tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The function call information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<DeltaFunctionCall>,
}

/// A function call in a streamed delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaFunctionCall {
    /// The name of the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The arguments for the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// The delta of a streamed choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    /// The role of the delta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// The content of the delta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// The tool calls in the delta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<DeltaToolCall>>,
}

/// A response from an LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    /// The ID of the response.
    pub id: String,
    /// The object type.
    pub object: String,
    /// The creation timestamp.
    pub created: u64,
    /// The model that generated the response.
    pub model: String,
    /// The choices in the response.
    pub choices: Vec<Choice>,
    /// The usage statistics for the response.
    pub usage: Usage,
}

/// A choice in an LLM response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// The index of the choice.
    pub index: u32,
    /// The message of the choice.
    pub message: ChatMessage,
    /// The reason the generation finished.
    pub finish_reason: Option<String>,
}

/// The usage statistics for an LLM response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// The number of tokens in the prompt.
    pub prompt_tokens: u32,
    /// The number of tokens in the completion.
    pub completion_tokens: u32,
    /// The total number of tokens.
    pub total_tokens: u32,
}

/// A trait for LLM providers.
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Generates a response from the LLM.
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse>;
    /// Returns the provider as an `Any` type.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// A client for interacting with an LLM.
pub struct LLMClient {
    provider: Box<dyn LLMProvider + Send + Sync>,
    provider_type: LLMProviderType,
}

impl LLMClient {
    /// Creates a new `LLMClient`.
    pub async fn new(provider_type: LLMProviderType) -> Result<Self> {
        let provider: Box<dyn LLMProvider + Send + Sync> = match &provider_type {
            LLMProviderType::Remote(config) => Box::new(RemoteLLMClient::new(config.clone())),
            #[cfg(feature = "local")]
            LLMProviderType::Local(config) => {
                Box::new(LocalLLMProvider::new(config.clone()).await?)
            }
            #[cfg(feature = "candle")]
            LLMProviderType::Candle(config) => {
                Box::new(CandleLLMProvider::new(config.clone()).await?)
            }
        };

        Ok(Self {
            provider,
            provider_type,
        })
    }

    /// Returns the type of the LLM provider.
    pub fn provider_type(&self) -> &LLMProviderType {
        &self.provider_type
    }
}

/// A client for interacting with a remote LLM.
pub struct RemoteLLMClient {
    config: LLMConfig,
    client: Client,
}

impl RemoteLLMClient {
    /// Creates a new `RemoteLLMClient`.
    pub fn new(config: LLMConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Returns the configuration of the client.
    pub fn config(&self) -> &LLMConfig {
        &self.config
    }
}

/// Suppresses stdout and stderr.
#[cfg(feature = "local")]
fn suppress_output() -> (i32, i32) {
    // Open /dev/null for writing
    let dev_null = File::open("/dev/null").expect("Failed to open /dev/null");

    // Duplicate current stdout and stderr file descriptors
    let stdout_backup = unsafe { libc::dup(1) };
    let stderr_backup = unsafe { libc::dup(2) };

    // Redirect stdout and stderr to /dev/null
    unsafe {
        libc::dup2(dev_null.as_raw_fd(), 1); // stdout
        libc::dup2(dev_null.as_raw_fd(), 2); // stderr
    }

    (stdout_backup, stderr_backup)
}

/// Restores stdout and stderr.
#[cfg(feature = "local")]
fn restore_output(stdout_backup: i32, stderr_backup: i32) {
    unsafe {
        libc::dup2(stdout_backup, 1); // restore stdout
        libc::dup2(stderr_backup, 2); // restore stderr
        libc::close(stdout_backup);
        libc::close(stderr_backup);
    }
}

/// Suppresses stderr.
#[cfg(feature = "local")]
fn suppress_stderr() -> i32 {
    let dev_null = File::open("/dev/null").expect("Failed to open /dev/null");
    let stderr_backup = unsafe { libc::dup(2) };
    unsafe {
        libc::dup2(dev_null.as_raw_fd(), 2);
    }
    stderr_backup
}

/// Restores stderr.
#[cfg(feature = "local")]
fn restore_stderr(stderr_backup: i32) {
    unsafe {
        libc::dup2(stderr_backup, 2);
        libc::close(stderr_backup);
    }
}

/// A provider for a local LLM.
#[cfg(feature = "local")]
pub struct LocalLLMProvider {
    model: Arc<LlamaModel>,
    backend: Arc<LlamaBackend>,
}

#[cfg(feature = "local")]
impl LocalLLMProvider {
    /// Creates a new `LocalLLMProvider`.
    pub async fn new(config: LocalConfig) -> Result<Self> {
        // Suppress verbose output during model loading in offline mode
        let (stdout_backup, stderr_backup) = suppress_output();

        // Initialize llama backend
        let backend = LlamaBackend::init().map_err(|e| {
            restore_output(stdout_backup, stderr_backup);
            HeliosError::LLMError(format!("Failed to initialize llama backend: {:?}", e))
        })?;

        // Download model from HuggingFace if needed
        let model_path = Self::download_model(&config).await.map_err(|e| {
            restore_output(stdout_backup, stderr_backup);
            e
        })?;

        // Load the model
        let model_params = LlamaModelParams::default().with_n_gpu_layers(99); // Use GPU if available

        let model =
            LlamaModel::load_from_file(&backend, &model_path, &model_params).map_err(|e| {
                restore_output(stdout_backup, stderr_backup);
                HeliosError::LLMError(format!("Failed to load model: {:?}", e))
            })?;

        // Restore output
        restore_output(stdout_backup, stderr_backup);

        Ok(Self {
            model: Arc::new(model),
            backend: Arc::new(backend),
        })
    }

    /// Downloads a model from Hugging Face.
    async fn download_model(config: &LocalConfig) -> Result<std::path::PathBuf> {
        use std::process::Command;

        // Check if model is already in HuggingFace cache
        if let Some(cached_path) =
            Self::find_model_in_cache(&config.huggingface_repo, &config.model_file)
        {
            // Model found in cache - no output needed in offline mode
            return Ok(cached_path);
        }

        // Model not found in cache - suppress download output in offline mode

        // Use huggingface_hub to download the model (suppress output)
        let output = Command::new("huggingface-cli")
            .args([
                "download",
                &config.huggingface_repo,
                &config.model_file,
                "--local-dir",
                ".cache/models",
                "--local-dir-use-symlinks",
                "False",
            ])
            .stdout(std::process::Stdio::null()) // Suppress stdout
            .stderr(std::process::Stdio::null()) // Suppress stderr
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

    /// Finds a model in the Hugging Face cache.
    fn find_model_in_cache(repo: &str, model_file: &str) -> Option<std::path::PathBuf> {
        // Check HuggingFace cache directory
        let cache_dir = std::env::var("HF_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                std::path::PathBuf::from(home)
                    .join(".cache")
                    .join("huggingface")
            });

        let hub_dir = cache_dir.join("hub");

        // Convert repo name to HuggingFace cache format
        // e.g., "unsloth/Qwen3-0.6B-GGUF" -> "models--unsloth--Qwen3-0.6B-GGUF"
        let cache_repo_name = format!("models--{}", repo.replace("/", "--"));
        let repo_dir = hub_dir.join(&cache_repo_name);

        if !repo_dir.exists() {
            return None;
        }

        // Check in snapshots directory (newer cache format)
        let snapshots_dir = repo_dir.join("snapshots");
        if snapshots_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
                for entry in entries.flatten() {
                    if let Ok(snapshot_path) = entry.path().join(model_file).canonicalize() {
                        if snapshot_path.exists() {
                            return Some(snapshot_path);
                        }
                    }
                }
            }
        }

        // Check in blobs directory (alternative cache format)
        let blobs_dir = repo_dir.join("blobs");
        if blobs_dir.exists() {
            // For blobs, we need to find the blob file by hash
            // This is more complex, so for now we'll skip this check
            // The snapshots approach should cover most cases
        }

        None
    }

    /// Formats a list of messages into a single string.
    fn format_messages(&self, messages: &[ChatMessage]) -> String {
        let mut formatted = String::new();

        // Use Qwen chat template format
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
                    // For tool messages, include them as assistant responses
                    formatted.push_str("<|im_start|>assistant\n");
                    formatted.push_str(&message.content);
                    formatted.push_str("\n<|im_end|>\n");
                }
            }
        }

        // Start the assistant's response
        formatted.push_str("<|im_start|>assistant\n");

        formatted
    }
}

#[async_trait]
impl LLMProvider for RemoteLLMClient {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse> {
        let url = format!("{}/chat/completions", self.config.base_url);

        let mut request_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Only add authorization header if not a local vLLM instance
        if !self.config.base_url.contains("10.")
            && !self.config.base_url.contains("localhost")
            && !self.config.base_url.contains("127.0.0.1")
        {
            request_builder =
                request_builder.header("Authorization", format!("Bearer {}", self.config.api_key));
        }

        let response = request_builder.json(&request).send().await?;

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
    /// Sends a chat request to the remote LLM.
    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        stop: Option<Vec<String>>,
    ) -> Result<ChatMessage> {
        let request = LLMRequest {
            model: self.config.model_name.clone(),
            messages,
            temperature: temperature.or(Some(self.config.temperature)),
            max_tokens: max_tokens.or(Some(self.config.max_tokens)),
            tools: tools.clone(),
            tool_choice: if tools.is_some() {
                Some("auto".to_string())
            } else {
                None
            },
            stream: None,
            stop,
        };

        let response = self.generate(request).await?;

        response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message)
            .ok_or_else(|| HeliosError::LLMError("No response from LLM".to_string()))
    }

    /// Sends a streaming chat request to the remote LLM.
    pub async fn chat_stream<F>(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        stop: Option<Vec<String>>,
        mut on_chunk: F,
    ) -> Result<ChatMessage>
    where
        F: FnMut(&str) + Send,
    {
        let request = LLMRequest {
            model: self.config.model_name.clone(),
            messages,
            temperature: temperature.or(Some(self.config.temperature)),
            max_tokens: max_tokens.or(Some(self.config.max_tokens)),
            tools: tools.clone(),
            tool_choice: if tools.is_some() {
                Some("auto".to_string())
            } else {
                None
            },
            stream: Some(true),
            stop,
        };

        let url = format!("{}/chat/completions", self.config.base_url);

        let mut request_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Only add authorization header if not a local vLLM instance
        if !self.config.base_url.contains("10.")
            && !self.config.base_url.contains("localhost")
            && !self.config.base_url.contains("127.0.0.1")
        {
            request_builder =
                request_builder.header("Authorization", format!("Bearer {}", self.config.api_key));
        }

        let response = request_builder.json(&request).send().await?;

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
        let mut tool_calls = Vec::new();
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
                                if let Some(delta_tool_calls) = &choice.delta.tool_calls {
                                    for delta_tool_call in delta_tool_calls {
                                        // Find or create the tool call at this index
                                        while tool_calls.len() <= delta_tool_call.index as usize {
                                            tool_calls.push(None);
                                        }
                                        let tool_call_slot =
                                            &mut tool_calls[delta_tool_call.index as usize];

                                        if tool_call_slot.is_none() {
                                            *tool_call_slot = Some(crate::chat::ToolCall {
                                                id: String::new(),
                                                call_type: "function".to_string(),
                                                function: crate::chat::FunctionCall {
                                                    name: String::new(),
                                                    arguments: String::new(),
                                                },
                                            });
                                        }

                                        if let Some(tool_call) = tool_call_slot.as_mut() {
                                            if let Some(id) = &delta_tool_call.id {
                                                tool_call.id = id.clone();
                                            }
                                            if let Some(function) = &delta_tool_call.function {
                                                if let Some(name) = &function.name {
                                                    tool_call.function.name = name.clone();
                                                }
                                                if let Some(args) = &function.arguments {
                                                    tool_call.function.arguments.push_str(args);
                                                }
                                            }
                                        }
                                    }
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

        let final_tool_calls = tool_calls.into_iter().flatten().collect::<Vec<_>>();
        let tool_calls_option = if final_tool_calls.is_empty() {
            None
        } else {
            Some(final_tool_calls)
        };

        Ok(ChatMessage {
            role: crate::chat::Role::from(role.as_deref().unwrap_or("assistant")),
            content: full_content,
            name: None,
            tool_calls: tool_calls_option,
            tool_call_id: None,
        })
    }
}

#[cfg(feature = "local")]
#[async_trait]
impl LLMProvider for LocalLLMProvider {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse> {
        let prompt = self.format_messages(&request.messages);

        // Suppress output during inference in offline mode
        let (stdout_backup, stderr_backup) = suppress_output();

        // Run inference in a blocking task
        let model = Arc::clone(&self.model);
        let backend = Arc::clone(&self.backend);
        let result = task::spawn_blocking(move || {
            // Create a fresh context per request (model/back-end are reused across calls)
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
            let max_new_tokens = 512; // Increased limit for better responses
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
                match context.model.token_to_str(token, Special::Plaintext) {
                    Ok(text) => {
                        generated_text.push_str(&text);
                    }
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
        .map_err(|e| {
            restore_output(stdout_backup, stderr_backup);
            HeliosError::LLMError(format!("Task failed: {}", e))
        })??;

        // Restore output after inference completes
        restore_output(stdout_backup, stderr_backup);

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

#[cfg(feature = "local")]
impl LocalLLMProvider {
    /// Sends a streaming chat request to the local LLM.
    async fn chat_stream_local<F>(
        &self,
        messages: Vec<ChatMessage>,
        _temperature: Option<f32>,
        _max_tokens: Option<u32>,
        _stop: Option<Vec<String>>,
        mut on_chunk: F,
    ) -> Result<ChatMessage>
    where
        F: FnMut(&str) + Send,
    {
        let prompt = self.format_messages(&messages);

        // Suppress only stderr so llama.cpp context logs are hidden but stdout streaming remains visible
        let stderr_backup = suppress_stderr();

        // Create a channel for streaming tokens
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        // Spawn blocking task for generation
        let model = Arc::clone(&self.model);
        let backend = Arc::clone(&self.backend);
        let generation_task = task::spawn_blocking(move || {
            // Create a fresh context per request (model/back-end are reused across calls)
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
                let compute_logits = true;
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

            // Generate response tokens with streaming
            let mut generated_text = String::new();
            let max_new_tokens = 512;
            let mut next_pos = tokens.len() as i32;

            for _ in 0..max_new_tokens {
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
                match context.model.token_to_str(token, Special::Plaintext) {
                    Ok(text) => {
                        generated_text.push_str(&text);
                        // Send token through channel; stop if receiver is dropped
                        if tx.send(text).is_err() {
                            break;
                        }
                    }
                    Err(_) => continue,
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
        });

        // Receive and process tokens as they arrive
        while let Some(token) = rx.recv().await {
            on_chunk(&token);
        }

        // Wait for generation to complete and get the result
        let result = match generation_task.await {
            Ok(Ok(text)) => text,
            Ok(Err(e)) => {
                restore_stderr(stderr_backup);
                return Err(e);
            }
            Err(e) => {
                restore_stderr(stderr_backup);
                return Err(HeliosError::LLMError(format!("Task failed: {}", e)));
            }
        };

        // Restore stderr after generation completes
        restore_stderr(stderr_backup);

        Ok(ChatMessage {
            role: crate::chat::Role::Assistant,
            content: result,
            name: None,
            tool_calls: None,
            tool_call_id: None,
        })
    }
}

#[async_trait]
impl LLMProvider for LLMClient {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse> {
        self.provider.generate(request).await
    }
}

impl LLMClient {
    /// Sends a chat request to the LLM.
    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        stop: Option<Vec<String>>,
    ) -> Result<ChatMessage> {
        let (model_name, default_temperature, default_max_tokens) = match &self.provider_type {
            LLMProviderType::Remote(config) => (
                config.model_name.clone(),
                config.temperature,
                config.max_tokens,
            ),
            #[cfg(feature = "local")]
            LLMProviderType::Local(config) => (
                "local-model".to_string(),
                config.temperature,
                config.max_tokens,
            ),
            #[cfg(feature = "candle")]
            LLMProviderType::Candle(config) => (
                config.huggingface_repo.clone(),
                config.temperature,
                config.max_tokens,
            ),
        };

        let request = LLMRequest {
            model: model_name,
            messages,
            temperature: temperature.or(Some(default_temperature)),
            max_tokens: max_tokens.or(Some(default_max_tokens)),
            tools: tools.clone(),
            tool_choice: if tools.is_some() {
                Some("auto".to_string())
            } else {
                None
            },
            stream: None,
            stop,
        };

        let response = self.generate(request).await?;

        response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message)
            .ok_or_else(|| HeliosError::LLMError("No response from LLM".to_string()))
    }

    /// Sends a streaming chat request to the LLM.
    pub async fn chat_stream<F>(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        stop: Option<Vec<String>>,
        on_chunk: F,
    ) -> Result<ChatMessage>
    where
        F: FnMut(&str) + Send,
    {
        match &self.provider_type {
            LLMProviderType::Remote(_) => {
                if let Some(provider) = self.provider.as_any().downcast_ref::<RemoteLLMClient>() {
                    provider
                        .chat_stream(messages, tools, temperature, max_tokens, stop, on_chunk)
                        .await
                } else {
                    Err(HeliosError::AgentError("Provider type mismatch".into()))
                }
            }
            #[cfg(feature = "local")]
            LLMProviderType::Local(_) => {
                if let Some(provider) = self.provider.as_any().downcast_ref::<LocalLLMProvider>() {
                    provider
                        .chat_stream_local(messages, temperature, max_tokens, stop, on_chunk)
                        .await
                } else {
                    Err(HeliosError::AgentError("Provider type mismatch".into()))
                }
            }
            #[cfg(feature = "candle")]
            LLMProviderType::Candle(config) => {
                // For Candle, use non-streaming generate and call on_chunk with full response
                let (model_name, default_temperature, default_max_tokens) = (
                    config.huggingface_repo.clone(),
                    config.temperature,
                    config.max_tokens,
                );

                let request = LLMRequest {
                    model: model_name,
                    messages,
                    temperature: temperature.or(Some(default_temperature)),
                    max_tokens: max_tokens.or(Some(default_max_tokens)),
                    tools: tools.clone(),
                    tool_choice: if tools.is_some() {
                        Some("auto".to_string())
                    } else {
                        None
                    },
                    stream: None,
                    stop,
                };

                let response = self.provider.generate(request).await?;
                if let Some(choice) = response.choices.first() {
                    on_chunk(&choice.message.content);
                }
                response
                    .choices
                    .into_iter()
                    .next()
                    .map(|choice| choice.message)
                    .ok_or_else(|| HeliosError::LLMError("No response from LLM".to_string()))
            }
        }
    }
}

// Test module added
