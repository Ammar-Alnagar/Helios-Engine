use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
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
        };

        let response = self.generate(request).await?;
        
        response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message)
            .ok_or_else(|| HeliosError::LLMError("No response from LLM".to_string()))
    }

    pub async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatMessage> {
        // For simplicity, using non-streaming version
        // Streaming can be implemented with server-sent events
        self.chat(messages, None).await
    }
}
