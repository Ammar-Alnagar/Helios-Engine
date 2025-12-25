//! # RAG Tool Implementation
//!
//! Provides a Tool implementation that wraps the RAG system for agent use.

use crate::error::{HeliosError, Result};
use crate::rag::{
    InMemoryVectorStore, OpenAIEmbeddings, QdrantVectorStore, RAGSystem, SearchResult,
};
use crate::tools::{Tool, ToolParameter, ToolResult};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// RAG Tool with flexible backend support
#[derive(Clone)]
pub struct RAGTool {
    rag_system: std::sync::Arc<RAGSystem>,
    backend_type: String,
}

impl RAGTool {
    /// Create a new RAG tool with in-memory vector store
    pub fn new_in_memory(
        embedding_api_url: impl Into<String>,
        embedding_api_key: impl Into<String>,
    ) -> Self {
        let embeddings = OpenAIEmbeddings::new(embedding_api_url, embedding_api_key);
        let vector_store = InMemoryVectorStore::new();
        let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

        Self {
            rag_system: std::sync::Arc::new(rag_system),
            backend_type: "in-memory".to_string(),
        }
    }

    /// Create a new RAG tool with Qdrant vector store
    pub fn new_qdrant(
        qdrant_url: impl Into<String>,
        collection_name: impl Into<String>,
        embedding_api_url: impl Into<String>,
        embedding_api_key: impl Into<String>,
    ) -> Self {
        let embeddings = OpenAIEmbeddings::new(embedding_api_url, embedding_api_key);
        let vector_store = QdrantVectorStore::new(qdrant_url, collection_name);
        let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

        Self {
            rag_system: std::sync::Arc::new(rag_system),
            backend_type: "qdrant".to_string(),
        }
    }

    /// Create with a custom RAG system
    pub fn with_rag_system(rag_system: RAGSystem, backend_type: impl Into<String>) -> Self {
        Self {
            rag_system: std::sync::Arc::new(rag_system),
            backend_type: backend_type.into(),
        }
    }

    /// Format search results for display
    fn format_results(&self, results: &[SearchResult]) -> String {
        if results.is_empty() {
            return "No matching documents found".to_string();
        }

        let formatted_results: Vec<String> = results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let preview = if result.text.len() > 200 {
                    format!("{}...", &result.text[..200])
                } else {
                    result.text.clone()
                };

                format!(
                    "{}. [Score: {:.4}] {}\n   ID: {}",
                    i + 1,
                    result.score,
                    preview,
                    result.id
                )
            })
            .collect();

        format!(
            "Found {} result(s):\n\n{}",
            results.len(),
            formatted_results.join("\n\n")
        )
    }
}

#[async_trait]
impl Tool for RAGTool {
    fn name(&self) -> &str {
        "rag"
    }

    fn description(&self) -> &str {
        "RAG (Retrieval-Augmented Generation) tool for document storage and semantic search. \
         Operations: add_document, search, delete, clear, count"
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "operation".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Operation: 'add_document', 'search', 'delete', 'clear', 'count'"
                    .to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "text".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Text content for add_document or search query".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "doc_id".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Document ID for delete operation".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "limit".to_string(),
            ToolParameter {
                param_type: "number".to_string(),
                description: "Number of results for search (default: 5)".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "metadata".to_string(),
            ToolParameter {
                param_type: "object".to_string(),
                description: "Additional metadata for the document (JSON object)".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'operation' parameter".to_string()))?;

        match operation {
            "add_document" => {
                let text = args.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
                    HeliosError::ToolError("Missing 'text' for add_document".to_string())
                })?;

                let metadata: Option<HashMap<String, serde_json::Value>> = args
                    .get("metadata")
                    .and_then(|v| serde_json::from_value(v.clone()).ok());

                let doc_id = self.rag_system.add_document(text, metadata).await?;

                let preview = if text.len() > 100 {
                    format!("{}...", &text[..100])
                } else {
                    text.to_string()
                };

                Ok(ToolResult::success(format!(
                    "✓ Document added successfully (backend: {})\nID: {}\nText preview: {}",
                    self.backend_type, doc_id, preview
                )))
            }
            "search" => {
                let query = args.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
                    HeliosError::ToolError("Missing 'text' for search".to_string())
                })?;

                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

                let results = self.rag_system.search(query, limit).await?;
                Ok(ToolResult::success(self.format_results(&results)))
            }
            "delete" => {
                let doc_id = args.get("doc_id").and_then(|v| v.as_str()).ok_or_else(|| {
                    HeliosError::ToolError("Missing 'doc_id' for delete".to_string())
                })?;

                self.rag_system.delete_document(doc_id).await?;
                Ok(ToolResult::success(format!(
                    "✓ Document '{}' deleted",
                    doc_id
                )))
            }
            "clear" => {
                self.rag_system.clear().await?;
                Ok(ToolResult::success(
                    "✓ All documents cleared from collection".to_string(),
                ))
            }
            "count" => {
                let count = self.rag_system.count().await?;
                Ok(ToolResult::success(format!(
                    "Document count: {} (backend: {})",
                    count, self.backend_type
                )))
            }
            _ => Err(HeliosError::ToolError(format!(
                "Unknown operation '{}'. Valid: add_document, search, delete, clear, count",
                operation
            ))),
        }
    }
}
