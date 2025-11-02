//! # RAG (Retrieval-Augmented Generation) Module
//!
//! This module provides a flexible RAG system with:
//! - Multiple vector store backends (in-memory, Qdrant)
//! - Embedding generation (OpenAI API, local models)
//! - Document chunking and preprocessing
//! - Semantic search and retrieval
//! - Reranking capabilities

use crate::error::{HeliosError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Core Types and Traits
// ============================================================================

/// Represents a document in the RAG system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique document identifier
    pub id: String,
    /// The text content of the document
    pub text: String,
    /// Optional metadata associated with the document
    pub metadata: HashMap<String, serde_json::Value>,
    /// Timestamp when the document was added
    pub timestamp: String,
}

/// Represents a search result from the RAG system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document ID
    pub id: String,
    /// Similarity score (0.0 to 1.0, higher is better)
    pub score: f64,
    /// The document text
    pub text: String,
    /// Optional metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

// ============================================================================
// Embedding Provider Trait
// ============================================================================

/// Trait for embedding generation
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embeddings for the given text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Get the dimension of embeddings produced by this provider
    fn dimension(&self) -> usize;
}

// ============================================================================
// Vector Store Trait
// ============================================================================

/// Trait for vector storage backends
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Initialize the vector store (create collections, etc.)
    async fn initialize(&self, dimension: usize) -> Result<()>;

    /// Add a document with its embedding
    async fn add(
        &self,
        id: &str,
        embedding: Vec<f32>,
        text: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<()>;

    /// Search for similar documents
    async fn search(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>>;

    /// Delete a document by ID
    async fn delete(&self, id: &str) -> Result<()>;

    /// Clear all documents
    async fn clear(&self) -> Result<()>;

    /// Get document count
    async fn count(&self) -> Result<usize>;
}

// ============================================================================
// OpenAI Embedding Provider
// ============================================================================

/// OpenAI embedding provider using text-embedding-ada-002 or text-embedding-3-small
pub struct OpenAIEmbeddings {
    api_url: String,
    api_key: String,
    model: String,
    client: Client,
}

#[derive(Debug, Serialize)]
struct OpenAIEmbeddingRequest {
    input: String,
    model: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
}

impl OpenAIEmbeddings {
    /// Create a new OpenAI embeddings provider
    pub fn new(api_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            api_url: api_url.into(),
            api_key: api_key.into(),
            model: "text-embedding-ada-002".to_string(),
            client: Client::new(),
        }
    }

    /// Create with a specific model
    pub fn with_model(
        api_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            api_url: api_url.into(),
            api_key: api_key.into(),
            model: model.into(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddings {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let request = OpenAIEmbeddingRequest {
            input: text.to_string(),
            model: self.model.clone(),
        };

        let response = self
            .client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Embedding API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!(
                "Embedding API failed: {}",
                error_text
            )));
        }

        let embedding_response: OpenAIEmbeddingResponse = response.json().await.map_err(|e| {
            HeliosError::ToolError(format!("Failed to parse embedding response: {}", e))
        })?;

        embedding_response
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| HeliosError::ToolError("No embedding returned".to_string()))
    }

    fn dimension(&self) -> usize {
        // text-embedding-ada-002 produces 1536-dimensional embeddings
        // text-embedding-3-small produces 1536 by default
        // text-embedding-3-large produces 3072 by default
        if self.model == "text-embedding-3-large" {
            3072
        } else {
            1536
        }
    }
}

// ============================================================================
// In-Memory Vector Store
// ============================================================================

/// In-memory vector store using cosine similarity
pub struct InMemoryVectorStore {
    documents: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, StoredDocument>>>,
}

#[derive(Debug, Clone)]
struct StoredDocument {
    id: String,
    embedding: Vec<f32>,
    text: String,
    metadata: HashMap<String, serde_json::Value>,
}

impl InMemoryVectorStore {
    /// Create a new in-memory vector store
    pub fn new() -> Self {
        Self {
            documents: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

impl Default for InMemoryVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    (dot_product / (norm_a * norm_b)) as f64
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn initialize(&self, _dimension: usize) -> Result<()> {
        // No initialization needed for in-memory store
        Ok(())
    }

    async fn add(
        &self,
        id: &str,
        embedding: Vec<f32>,
        text: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let mut docs = self.documents.write().await;

        // Remove existing document with same ID if present
        docs.retain(|doc| doc.id != id);

        docs.push(StoredDocument {
            id: id.to_string(),
            embedding,
            text: text.to_string(),
            metadata,
        });

        Ok(())
    }

    async fn search(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>> {
        let docs = self.documents.read().await;

        if docs.is_empty() {
            return Ok(Vec::new());
        }

        // Calculate similarities for all documents
        let mut results: Vec<(usize, f64)> = docs
            .iter()
            .enumerate()
            .map(|(idx, doc)| {
                let similarity = cosine_similarity(&query_embedding, &doc.embedding);
                (idx, similarity)
            })
            .collect();

        // Sort by similarity (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top results
        let top_results: Vec<SearchResult> = results
            .into_iter()
            .take(limit)
            .map(|(idx, score)| {
                let doc = &docs[idx];
                SearchResult {
                    id: doc.id.clone(),
                    score,
                    text: doc.text.clone(),
                    metadata: Some(doc.metadata.clone()),
                }
            })
            .collect();

        Ok(top_results)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let mut docs = self.documents.write().await;
        docs.retain(|doc| doc.id != id);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut docs = self.documents.write().await;
        docs.clear();
        Ok(())
    }

    async fn count(&self) -> Result<usize> {
        let docs = self.documents.read().await;
        Ok(docs.len())
    }
}

// ============================================================================
// Qdrant Vector Store
// ============================================================================

/// Qdrant vector store implementation
pub struct QdrantVectorStore {
    qdrant_url: String,
    collection_name: String,
    client: Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct QdrantPoint {
    id: String,
    vector: Vec<f32>,
    payload: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QdrantSearchRequest {
    vector: Vec<f32>,
    limit: usize,
    with_payload: bool,
    with_vector: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct QdrantSearchResponse {
    result: Vec<QdrantSearchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QdrantSearchResult {
    id: String,
    score: f64,
    payload: Option<HashMap<String, serde_json::Value>>,
}

impl QdrantVectorStore {
    /// Create a new Qdrant vector store
    pub fn new(qdrant_url: impl Into<String>, collection_name: impl Into<String>) -> Self {
        Self {
            qdrant_url: qdrant_url.into(),
            collection_name: collection_name.into(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl VectorStore for QdrantVectorStore {
    async fn initialize(&self, dimension: usize) -> Result<()> {
        let collection_url = format!("{}/collections/{}", self.qdrant_url, self.collection_name);

        // Check if collection exists
        let response = self.client.get(&collection_url).send().await;

        if response.is_ok() && response.unwrap().status().is_success() {
            return Ok(()); // Collection exists
        }

        // Create collection
        let create_payload = serde_json::json!({
            "vectors": {
                "size": dimension,
                "distance": "Cosine"
            }
        });

        let response = self
            .client
            .put(&collection_url)
            .json(&create_payload)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Failed to create collection: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!(
                "Collection creation failed: {}",
                error_text
            )));
        }

        Ok(())
    }

    async fn add(
        &self,
        id: &str,
        embedding: Vec<f32>,
        text: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let mut payload = metadata;
        payload.insert("text".to_string(), serde_json::json!(text));
        payload.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now().to_rfc3339()),
        );

        let point = QdrantPoint {
            id: id.to_string(),
            vector: embedding,
            payload,
        };

        let upsert_url = format!(
            "{}/collections/{}/points",
            self.qdrant_url, self.collection_name
        );
        let upsert_payload = serde_json::json!({
            "points": [point]
        });

        let response = self
            .client
            .put(&upsert_url)
            .json(&upsert_payload)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Failed to upload document: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!(
                "Document upload failed: {}",
                error_text
            )));
        }

        Ok(())
    }

    async fn search(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>> {
        let search_url = format!(
            "{}/collections/{}/points/search",
            self.qdrant_url, self.collection_name
        );
        let search_request = QdrantSearchRequest {
            vector: query_embedding,
            limit,
            with_payload: true,
            with_vector: false,
        };

        let response = self
            .client
            .post(&search_url)
            .json(&search_request)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Search failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!(
                "Search request failed: {}",
                error_text
            )));
        }

        let search_response: QdrantSearchResponse = response.json().await.map_err(|e| {
            HeliosError::ToolError(format!("Failed to parse search response: {}", e))
        })?;

        let results: Vec<SearchResult> = search_response
            .result
            .into_iter()
            .filter_map(|r| {
                r.payload.and_then(|p| {
                    p.get("text").and_then(|t| t.as_str()).map(|text| {
                        let mut metadata = p.clone();
                        metadata.remove("text");
                        SearchResult {
                            id: r.id,
                            score: r.score,
                            text: text.to_string(),
                            metadata: Some(metadata),
                        }
                    })
                })
            })
            .collect();

        Ok(results)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let delete_url = format!(
            "{}/collections/{}/points/delete",
            self.qdrant_url, self.collection_name
        );
        let delete_payload = serde_json::json!({
            "points": [id]
        });

        let response = self
            .client
            .post(&delete_url)
            .json(&delete_payload)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Delete failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!(
                "Delete request failed: {}",
                error_text
            )));
        }

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let delete_url = format!("{}/collections/{}", self.qdrant_url, self.collection_name);

        let response = self
            .client
            .delete(&delete_url)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Clear failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!(
                "Clear collection failed: {}",
                error_text
            )));
        }

        Ok(())
    }

    async fn count(&self) -> Result<usize> {
        let count_url = format!("{}/collections/{}", self.qdrant_url, self.collection_name);

        let response = self
            .client
            .get(&count_url)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Count failed: {}", e)))?;

        if !response.status().is_success() {
            return Ok(0);
        }

        // Parse collection info to get count
        let info: serde_json::Value = response.json().await.map_err(|e| {
            HeliosError::ToolError(format!("Failed to parse collection info: {}", e))
        })?;

        let count = info
            .get("result")
            .and_then(|r| r.get("points_count"))
            .and_then(|c| c.as_u64())
            .unwrap_or(0) as usize;

        Ok(count)
    }
}

// ============================================================================
// RAG System
// ============================================================================

/// Main RAG system that combines embedding provider and vector store
pub struct RAGSystem {
    embedding_provider: Box<dyn EmbeddingProvider>,
    vector_store: Box<dyn VectorStore>,
    initialized: std::sync::Arc<tokio::sync::RwLock<bool>>,
}

impl RAGSystem {
    /// Create a new RAG system
    pub fn new(
        embedding_provider: Box<dyn EmbeddingProvider>,
        vector_store: Box<dyn VectorStore>,
    ) -> Self {
        Self {
            embedding_provider,
            vector_store,
            initialized: std::sync::Arc::new(tokio::sync::RwLock::new(false)),
        }
    }

    /// Ensure the system is initialized
    async fn ensure_initialized(&self) -> Result<()> {
        let mut initialized = self.initialized.write().await;
        if !*initialized {
            let dimension = self.embedding_provider.dimension();
            self.vector_store.initialize(dimension).await?;
            *initialized = true;
        }
        Ok(())
    }

    /// Add a document to the RAG system
    pub async fn add_document(
        &self,
        text: &str,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<String> {
        self.ensure_initialized().await?;

        let id = Uuid::new_v4().to_string();
        let embedding = self.embedding_provider.embed(text).await?;

        let mut meta = metadata.unwrap_or_default();
        meta.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now().to_rfc3339()),
        );

        self.vector_store.add(&id, embedding, text, meta).await?;

        Ok(id)
    }

    /// Search for similar documents
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        self.ensure_initialized().await?;

        let query_embedding = self.embedding_provider.embed(query).await?;
        self.vector_store.search(query_embedding, limit).await
    }

    /// Delete a document by ID
    pub async fn delete_document(&self, id: &str) -> Result<()> {
        self.vector_store.delete(id).await
    }

    /// Clear all documents
    pub async fn clear(&self) -> Result<()> {
        self.vector_store.clear().await
    }

    /// Get document count
    pub async fn count(&self) -> Result<usize> {
        self.vector_store.count().await
    }
}
