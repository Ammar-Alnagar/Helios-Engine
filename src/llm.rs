use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use futures::stream::StreamExt;
use crate::chat::ChatMessage;
use crate::config::LLMConfig;
use crate::error::{HeliosError, Result};
use crate::tools::ToolDefinition;

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
    config: LLMConfig,
    client: Client,
}

impl LLMClient {
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

#[async_trait]
impl LLMProvider for LLMClient {
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
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::LLMError(format!(
                "LLM API request failed with status {}: {}",
                status, error_text
            )));
        }

        let llm_response: LLMResponse = response.json().await?;
        Ok(llm_response)
    }
}

impl LLMClient {
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
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
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
