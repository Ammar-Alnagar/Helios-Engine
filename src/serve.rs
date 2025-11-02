//! # Serve Module
//!
//! This module provides functionality to serve fully OpenAI-compatible API endpoints
//! with real-time streaming and parameter control, allowing users to expose their
//! agents or LLM clients via HTTP with full generation parameter support.
//!
//! ## Usage
//!
//! ### From CLI
//! ```bash
//! helios-engine serve --port 8000
//! ```
//!
//! ### Programmatically
//! ```no_run
//! use helios_engine::{Config, serve};
//!
//! #[tokio::main]
//! async fn main() -> helios_engine::Result<()> {
//!     let config = Config::from_file("config.toml")?;
//!     serve::start_server(config, "127.0.0.1:8000").await?;
//!     Ok(())
//! }
//! ```

use crate::agent::Agent;
use crate::chat::{ChatMessage, Role};
use crate::config::Config;
use crate::error::{HeliosError, Result};
use crate::llm::{LLMClient, LLMProviderType};
use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use uuid::Uuid;

/// OpenAI-compatible chat completion request.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChatCompletionRequest {
    /// The model to use.
    pub model: String,
    /// The messages to send.
    pub messages: Vec<OpenAIMessage>,
    /// The temperature to use.
    #[serde(default)]
    pub temperature: Option<f32>,
    /// The maximum number of tokens to generate.
    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// Whether to stream the response.
    #[serde(default)]
    pub stream: Option<bool>,
    /// Stop sequences.
    #[serde(default)]
    pub stop: Option<Vec<String>>,
}

/// OpenAI-compatible message format.
#[derive(Debug, Deserialize)]
pub struct OpenAIMessage {
    /// The role of the message sender.
    pub role: String,
    /// The content of the message.
    pub content: String,
    /// The name of the message sender (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// OpenAI-compatible chat completion response.
#[derive(Debug, Serialize)]
pub struct ChatCompletionResponse {
    /// The ID of the completion.
    pub id: String,
    /// The object type.
    pub object: String,
    /// The creation timestamp.
    pub created: u64,
    /// The model used.
    pub model: String,
    /// The choices in the response.
    pub choices: Vec<CompletionChoice>,
    /// Usage statistics.
    pub usage: Usage,
}

/// A choice in a completion response.
#[derive(Debug, Serialize)]
pub struct CompletionChoice {
    /// The index of the choice.
    pub index: u32,
    /// The message in the choice.
    pub message: OpenAIMessageResponse,
    /// The finish reason.
    pub finish_reason: String,
}

/// A message in a completion response.
#[derive(Debug, Serialize)]
pub struct OpenAIMessageResponse {
    /// The role of the message sender.
    pub role: String,
    /// The content of the message.
    pub content: String,
}

/// Usage statistics for a completion.
#[derive(Debug, Serialize)]
pub struct Usage {
    /// The number of prompt tokens.
    pub prompt_tokens: u32,
    /// The number of completion tokens.
    pub completion_tokens: u32,
    /// The total number of tokens.
    pub total_tokens: u32,
}

/// Model information for the models endpoint.
#[derive(Debug, Serialize)]
pub struct ModelInfo {
    /// The ID of the model.
    pub id: String,
    /// The object type.
    pub object: String,
    /// The creation timestamp.
    pub created: u64,
    /// The owner of the model.
    pub owned_by: String,
}

/// Models list response.
#[derive(Debug, Serialize)]
pub struct ModelsResponse {
    /// The object type.
    pub object: String,
    /// The list of models.
    pub data: Vec<ModelInfo>,
}

/// Custom endpoint configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct CustomEndpoint {
    /// The HTTP method (GET, POST, PUT, DELETE, PATCH).
    pub method: String,
    /// The endpoint path.
    pub path: String,
    /// The response body as JSON.
    pub response: serde_json::Value,
    /// Optional status code (defaults to 200).
    #[serde(default = "default_status_code")]
    pub status_code: u16,
}

fn default_status_code() -> u16 {
    200
}

/// Custom endpoints configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct CustomEndpointsConfig {
    /// List of custom endpoints.
    pub endpoints: Vec<CustomEndpoint>,
}

/// Server state containing the LLM client and agent (if any).
#[derive(Clone)]
pub struct ServerState {
    /// The LLM client for direct LLM calls.
    pub llm_client: Option<Arc<LLMClient>>,
    /// The agent (if serving an agent).
    pub agent: Option<Arc<RwLock<Agent>>>,
    /// The model name being served.
    pub model_name: String,
}

impl ServerState {
    /// Creates a new server state with an LLM client.
    pub fn with_llm_client(llm_client: LLMClient, model_name: String) -> Self {
        Self {
            llm_client: Some(Arc::new(llm_client)),
            agent: None,
            model_name,
        }
    }

    /// Creates a new server state with an agent.
    pub fn with_agent(agent: Agent, model_name: String) -> Self {
        Self {
            llm_client: None,
            agent: Some(Arc::new(RwLock::new(agent))),
            model_name,
        }
    }
}

/// Starts the HTTP server with the given configuration.
///
/// # Arguments
///
/// * `config` - The configuration to use for the LLM client.
/// * `address` - The address to bind to (e.g., "127.0.0.1:8000").
///
/// # Returns
///
/// A `Result` that resolves when the server shuts down.
pub async fn start_server(config: Config, address: &str) -> Result<()> {
    #[cfg(feature = "local")]
    let provider_type = if let Some(local_config) = config.local.clone() {
        LLMProviderType::Local(local_config)
    } else {
        LLMProviderType::Remote(config.llm.clone())
    };

    #[cfg(not(feature = "local"))]
    let provider_type = LLMProviderType::Remote(config.llm.clone());

    let llm_client = LLMClient::new(provider_type).await?;

    #[cfg(feature = "local")]
    let model_name = config
        .local
        .as_ref()
        .map(|_| "local-model".to_string())
        .unwrap_or_else(|| config.llm.model_name.clone());

    #[cfg(not(feature = "local"))]
    let model_name = config.llm.model_name.clone();

    let state = ServerState::with_llm_client(llm_client, model_name);

    let app = create_router(state);

    info!("🚀 Starting Helios Engine server on http://{}", address);
    info!("📡 OpenAI-compatible API endpoints:");
    info!("   POST /v1/chat/completions");
    info!("   GET  /v1/models");

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|e| HeliosError::ConfigError(format!("Failed to bind to {}: {}", address, e)))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| HeliosError::ConfigError(format!("Server error: {}", e)))?;

    Ok(())
}

/// Starts the HTTP server with an agent.
///
/// # Arguments
///
/// * `agent` - The agent to serve.
/// * `model_name` - The model name to expose in the API.
/// * `address` - The address to bind to (e.g., "127.0.0.1:8000").
///
/// # Returns
///
/// A `Result` that resolves when the server shuts down.
pub async fn start_server_with_agent(
    agent: Agent,
    model_name: String,
    address: &str,
) -> Result<()> {
    let state = ServerState::with_agent(agent, model_name);

    let app = create_router(state);

    info!(
        "🚀 Starting Helios Engine server with agent on http://{}",
        address
    );
    info!("📡 OpenAI-compatible API endpoints:");
    info!("   POST /v1/chat/completions");
    info!("   GET  /v1/models");

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|e| HeliosError::ConfigError(format!("Failed to bind to {}: {}", address, e)))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| HeliosError::ConfigError(format!("Server error: {}", e)))?;

    Ok(())
}

/// Starts the HTTP server with custom endpoints.
///
/// # Arguments
///
/// * `config` - The configuration to use for the LLM client.
/// * `address` - The address to bind to (e.g., "127.0.0.1:8000").
/// * `custom_endpoints` - Optional custom endpoints configuration.
///
/// # Returns
///
/// A `Result` that resolves when the server shuts down.
pub async fn start_server_with_custom_endpoints(
    config: Config,
    address: &str,
    custom_endpoints: Option<CustomEndpointsConfig>,
) -> Result<()> {
    #[cfg(feature = "local")]
    let provider_type = if let Some(local_config) = config.local.clone() {
        LLMProviderType::Local(local_config)
    } else {
        LLMProviderType::Remote(config.llm.clone())
    };

    #[cfg(not(feature = "local"))]
    let provider_type = LLMProviderType::Remote(config.llm.clone());

    let llm_client = LLMClient::new(provider_type).await?;

    #[cfg(feature = "local")]
    let model_name = config
        .local
        .as_ref()
        .map(|_| "local-model".to_string())
        .unwrap_or_else(|| config.llm.model_name.clone());

    #[cfg(not(feature = "local"))]
    let model_name = config.llm.model_name.clone();

    let state = ServerState::with_llm_client(llm_client, model_name);

    let app = create_router_with_custom_endpoints(state, custom_endpoints.clone());

    info!("🚀 Starting Helios Engine server on http://{}", address);
    info!("📡 OpenAI-compatible API endpoints:");
    info!("   POST /v1/chat/completions");
    info!("   GET  /v1/models");

    if let Some(config) = &custom_endpoints {
        info!("📡 Custom endpoints:");
        for endpoint in &config.endpoints {
            info!("   {} {}", endpoint.method.to_uppercase(), endpoint.path);
        }
    }

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|e| HeliosError::ConfigError(format!("Failed to bind to {}: {}", address, e)))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| HeliosError::ConfigError(format!("Server error: {}", e)))?;

    Ok(())
}

/// Starts the HTTP server with an agent and custom endpoints.
///
/// # Arguments
///
/// * `agent` - The agent to serve.
/// * `model_name` - The model name to expose in the API.
/// * `address` - The address to bind to (e.g., "127.0.0.1:8000").
/// * `custom_endpoints` - Optional custom endpoints configuration.
///
/// # Returns
///
/// A `Result` that resolves when the server shuts down.
pub async fn start_server_with_agent_and_custom_endpoints(
    agent: Agent,
    model_name: String,
    address: &str,
    custom_endpoints: Option<CustomEndpointsConfig>,
) -> Result<()> {
    let state = ServerState::with_agent(agent, model_name);

    let app = create_router_with_custom_endpoints(state, custom_endpoints.clone());

    info!(
        "🚀 Starting Helios Engine server with agent on http://{}",
        address
    );
    info!("📡 OpenAI-compatible API endpoints:");
    info!("   POST /v1/chat/completions");
    info!("   GET  /v1/models");

    if let Some(config) = &custom_endpoints {
        info!("📡 Custom endpoints:");
        for endpoint in &config.endpoints {
            info!("   {} {}", endpoint.method.to_uppercase(), endpoint.path);
        }
    }

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|e| HeliosError::ConfigError(format!("Failed to bind to {}: {}", address, e)))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| HeliosError::ConfigError(format!("Server error: {}", e)))?;

    Ok(())
}

/// Loads custom endpoints configuration from a TOML file.
///
/// # Arguments
///
/// * `path` - Path to the custom endpoints configuration file.
///
/// # Returns
///
/// A `Result` containing the custom endpoints configuration.
pub fn load_custom_endpoints_config(path: &str) -> Result<CustomEndpointsConfig> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        HeliosError::ConfigError(format!(
            "Failed to read custom endpoints config file '{}': {}",
            path, e
        ))
    })?;

    toml::from_str(&content).map_err(|e| {
        HeliosError::ConfigError(format!(
            "Failed to parse custom endpoints config file '{}': {}",
            path, e
        ))
    })
}

/// Creates the router with all endpoints.
fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Creates the router with custom endpoints.
fn create_router_with_custom_endpoints(
    state: ServerState,
    custom_endpoints: Option<CustomEndpointsConfig>,
) -> Router {
    let mut router = Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
        .route("/health", get(health_check));

    // Add custom endpoints if provided
    if let Some(config) = custom_endpoints {
        for endpoint in config.endpoints {
            let response = endpoint.response.clone();
            let status_code = StatusCode::from_u16(endpoint.status_code).unwrap_or(StatusCode::OK);

            let handler = move || async move { (status_code, Json(response)) };

            match endpoint.method.to_uppercase().as_str() {
                "GET" => router = router.route(&endpoint.path, get(handler)),
                "POST" => router = router.route(&endpoint.path, post(handler)),
                "PUT" => router = router.route(&endpoint.path, put(handler)),
                "DELETE" => router = router.route(&endpoint.path, delete(handler)),
                "PATCH" => router = router.route(&endpoint.path, patch(handler)),
                _ => {
                    // Default to GET for unsupported methods
                    router = router.route(&endpoint.path, get(handler));
                }
            }
        }
    }

    router
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Health check endpoint.
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "helios-engine"
    }))
}

/// Lists available models.
async fn list_models(State(state): State<ServerState>) -> Json<ModelsResponse> {
    Json(ModelsResponse {
        object: "list".to_string(),
        data: vec![ModelInfo {
            id: state.model_name.clone(),
            object: "model".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            owned_by: "helios-engine".to_string(),
        }],
    })
}

/// Handles chat completion requests.
async fn chat_completions(
    State(state): State<ServerState>,
    Json(request): Json<ChatCompletionRequest>,
) -> std::result::Result<impl axum::response::IntoResponse, StatusCode> {
    // Convert OpenAI messages to ChatMessage format
    let messages: Result<Vec<ChatMessage>> = request
        .messages
        .into_iter()
        .map(|msg| {
            // Convert OpenAI message format to internal ChatMessage format
            // Maps standard OpenAI roles to our Role enum
            let role = match msg.role.as_str() {
                "system" => Role::System,       // System instructions/prompts
                "user" => Role::User,           // User input messages
                "assistant" => Role::Assistant, // AI assistant responses
                "tool" => Role::Tool,           // Tool/function call results
                _ => {
                    // Reject invalid roles to maintain API compatibility
                    return Err(HeliosError::ConfigError(format!(
                        "Invalid role: {}",
                        msg.role
                    )));
                }
            };
            Ok(ChatMessage {
                role,
                content: msg.content, // The actual message text
                name: msg.name,       // Optional name for tool messages
                tool_calls: None,     // Not used in conversion (OpenAI format differs)
                tool_call_id: None,   // Not used in conversion (OpenAI format differs)
            })
        })
        .collect();

    let messages = messages.map_err(|e| {
        error!("Failed to convert messages: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    let stream = request.stream.unwrap_or(false);

    if stream {
        // Handle streaming response
        return Ok(stream_chat_completion(
            state,
            messages,
            request.model,
            request.temperature,
            request.max_tokens,
            request.stop.clone(),
        )
        .into_response());
    }

    // Handle non-streaming response
    let completion_id = format!("chatcmpl-{}", Uuid::new_v4());
    let created = chrono::Utc::now().timestamp() as u64;

    // Clone messages for token estimation and LLM client usage
    let messages_clone = messages.clone();

    let response_content = if let Some(agent) = &state.agent {
        // Use agent for response with full conversation history
        let mut agent = agent.write().await;

        match agent
            .chat_with_history(
                messages.clone(),
                request.temperature,
                request.max_tokens,
                request.stop.clone(),
            )
            .await
        {
            Ok(content) => content,
            Err(e) => {
                error!("Agent error: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else if let Some(llm_client) = &state.llm_client {
        // Use LLM client directly
        match llm_client
            .chat(
                messages_clone,
                None,
                request.temperature,
                request.max_tokens,
                request.stop.clone(),
            )
            .await
        {
            Ok(msg) => msg.content,
            Err(e) => {
                error!("LLM error: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    // Estimate token usage (simplified - in production, use actual tokenizer)
    let prompt_tokens = estimate_tokens(
        &messages
            .iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join(" "),
    );
    let completion_tokens = estimate_tokens(&response_content);

    let response = ChatCompletionResponse {
        id: completion_id,
        object: "chat.completion".to_string(),
        created,
        model: request.model,
        choices: vec![CompletionChoice {
            index: 0,
            message: OpenAIMessageResponse {
                role: "assistant".to_string(),
                content: response_content,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: Usage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        },
    };

    Ok(Json(response).into_response())
}

/// Streams a chat completion response.
fn stream_chat_completion(
    state: ServerState,
    messages: Vec<ChatMessage>,
    model: String,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stop: Option<Vec<String>>,
) -> Sse<impl Stream<Item = std::result::Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    let completion_id = format!("chatcmpl-{}", Uuid::new_v4());
    let created = chrono::Utc::now().timestamp() as u64;

    tokio::spawn(async move {
        let on_chunk = |chunk: &str| {
            let event = Event::default()
                .json_data(serde_json::json!({
                    "id": completion_id,
                    "object": "chat.completion.chunk",
                    "created": created,
                    "model": model,
                    "choices": [{
                        "index": 0,
                        "delta": {
                            "content": chunk
                        },
                        "finish_reason": null
                    }]
                }))
                .unwrap();
            let _ = tx.try_send(Ok(event));
        };

        if let Some(agent) = &state.agent {
            // Use agent for true streaming response with full conversation history
            let mut agent = agent.write().await;

            match agent
                .chat_stream_with_history(messages, temperature, max_tokens, stop.clone(), on_chunk)
                .await
            {
                Ok(_) => {
                    // Streaming completed successfully
                    // The on_chunk callback has already been called for each token
                }
                Err(e) => {
                    error!("Agent streaming error: {}", e);
                }
            }
        } else if let Some(llm_client) = &state.llm_client {
            // Use LLM client streaming
            match llm_client
                .chat_stream(
                    messages,
                    None,
                    temperature,
                    max_tokens,
                    stop.clone(),
                    on_chunk,
                )
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    error!("LLM streaming error: {}", e);
                }
            }
        };

        // Send final event
        let final_event = Event::default()
            .json_data(serde_json::json!({
                "id": completion_id,
                "object": "chat.completion.chunk",
                "created": created,
                "model": model,
                "choices": [{
                    "index": 0,
                    "delta": {},
                    "finish_reason": "stop"
                }]
            }))
            .unwrap();
        let _ = tx.send(Ok(final_event)).await;
    });

    Sse::new(ReceiverStream::new(rx)).keep_alive(axum::response::sse::KeepAlive::default())
}

/// Estimates the number of tokens in a text (simplified approximation).
/// In production, use an actual tokenizer.
pub fn estimate_tokens(text: &str) -> u32 {
    // Rough approximation: ~4 characters per token
    (text.len() as f32 / 4.0).ceil() as u32
}
