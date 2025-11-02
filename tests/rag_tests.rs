//! RAG system tests

use helios_engine::{InMemoryVectorStore, RAGSystem, VectorStore};
use std::collections::HashMap;

/// Mock embedding provider for testing
struct MockEmbeddings;

#[async_trait::async_trait]
impl helios_engine::EmbeddingProvider for MockEmbeddings {
    async fn embed(&self, text: &str) -> helios_engine::Result<Vec<f32>> {
        // Simple mock: convert text to a pattern
        let mut vec = vec![0.0; 128];

        // Create a unique pattern based on text content
        for (i, c) in text.chars().enumerate() {
            if i >= vec.len() {
                break;
            }
            vec[i] = (c as u32 as f32) / 1000.0;
        }

        // Normalize
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }

        Ok(vec)
    }

    fn dimension(&self) -> usize {
        128
    }
}

/// Mock embedding provider with configurable dimension
struct FixedDimensionEmbeddings {
    dimension: usize,
}

impl FixedDimensionEmbeddings {
    fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait::async_trait]
impl helios_engine::EmbeddingProvider for FixedDimensionEmbeddings {
    async fn embed(&self, text: &str) -> helios_engine::Result<Vec<f32>> {
        let mut vec = vec![0.0; self.dimension];

        for (i, c) in text.chars().enumerate() {
            if i >= vec.len() {
                break;
            }
            vec[i] = (c as u32 as f32) / 1000.0;
        }

        // Normalize
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }

        Ok(vec)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

#[tokio::test]
async fn test_in_memory_vector_store_add_and_search() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add documents
    let _id1 = rag_system
        .add_document("The quick brown fox jumps over the lazy dog", None)
        .await
        .unwrap();

    let _id2 = rag_system
        .add_document("A fast brown fox leaps over a sleepy dog", None)
        .await
        .unwrap();

    let _id3 = rag_system
        .add_document("Python is a programming language", None)
        .await
        .unwrap();

    // Search for similar documents
    let results = rag_system.search("quick brown fox", 2).await.unwrap();

    assert_eq!(results.len(), 2);
    // First result should be the most similar
    assert!(results[0].score > 0.5);
}

#[tokio::test]
async fn test_rag_system_count() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Initially empty
    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 0);

    // Add documents
    rag_system.add_document("Document 1", None).await.unwrap();
    rag_system.add_document("Document 2", None).await.unwrap();
    rag_system.add_document("Document 3", None).await.unwrap();

    // Check count
    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 3);
}

#[tokio::test]
async fn test_rag_system_delete() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add documents
    let id1 = rag_system.add_document("Document 1", None).await.unwrap();
    let id2 = rag_system.add_document("Document 2", None).await.unwrap();

    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 2);

    // Delete one document
    rag_system.delete_document(&id1).await.unwrap();

    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 1);

    // Search should only return remaining document
    let results = rag_system.search("Document", 10).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, id2);
}

#[tokio::test]
async fn test_rag_system_clear() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add documents
    rag_system.add_document("Document 1", None).await.unwrap();
    rag_system.add_document("Document 2", None).await.unwrap();
    rag_system.add_document("Document 3", None).await.unwrap();

    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 3);

    // Clear all
    rag_system.clear().await.unwrap();

    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 0);

    // Search should return empty
    let results = rag_system.search("Document", 10).await.unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_rag_system_with_metadata() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add document with metadata
    let mut metadata = HashMap::new();
    metadata.insert("category".to_string(), serde_json::json!("programming"));
    metadata.insert("language".to_string(), serde_json::json!("rust"));

    let _id = rag_system
        .add_document("Rust is a systems programming language", Some(metadata))
        .await
        .unwrap();

    // Search and verify metadata
    let results = rag_system.search("Rust programming", 1).await.unwrap();
    assert_eq!(results.len(), 1);

    let result = &results[0];
    assert!(result.metadata.is_some());

    let meta = result.metadata.as_ref().unwrap();
    assert_eq!(
        meta.get("category").and_then(|v| v.as_str()),
        Some("programming")
    );
    assert_eq!(meta.get("language").and_then(|v| v.as_str()), Some("rust"));
}

#[tokio::test]
async fn test_cosine_similarity() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add similar documents
    rag_system
        .add_document("The cat sat on the mat", None)
        .await
        .unwrap();

    rag_system
        .add_document("The cat sat on the rug", None)
        .await
        .unwrap();

    rag_system
        .add_document("Python programming language", None)
        .await
        .unwrap();

    // Search for similar document
    let results = rag_system
        .search("The cat sat on the mat", 3)
        .await
        .unwrap();

    // First result should be exact match (or very close)
    assert!(results[0].score > 0.8);

    // Second should be similar
    assert!(results[1].score > 0.5);

    // Third should be less similar
    assert!(results[2].score < results[1].score);
}

#[tokio::test]
async fn test_in_memory_store_add_and_get() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add document
    let _id = rag_system
        .add_document("Original text", None)
        .await
        .unwrap();

    // Count should be 1
    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 1);

    // Search to verify
    let results = rag_system.search("Original", 1).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].text, "Original text");
}

#[tokio::test]
async fn test_empty_search() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Search in empty store
    let results = rag_system.search("anything", 10).await.unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_search_limit() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add 10 documents
    for i in 0..10 {
        rag_system
            .add_document(&format!("Document number {}", i), None)
            .await
            .unwrap();
    }

    // Search with limit 5
    let results = rag_system.search("Document", 5).await.unwrap();
    assert_eq!(results.len(), 5);

    // Search with limit 3
    let results = rag_system.search("Document", 3).await.unwrap();
    assert_eq!(results.len(), 3);

    // Search with limit larger than document count
    let results = rag_system.search("Document", 20).await.unwrap();
    assert_eq!(results.len(), 10);
}

// ============================================================================
// Advanced RAG System Tests
// ============================================================================

#[tokio::test]
async fn test_multiple_identical_documents() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add identical documents
    let _id1 = rag_system.add_document("Same text", None).await.unwrap();
    let _id2 = rag_system.add_document("Same text", None).await.unwrap();
    let _id3 = rag_system.add_document("Same text", None).await.unwrap();

    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 3);

    // Search should return all with similar scores
    let results = rag_system.search("Same text", 3).await.unwrap();
    assert_eq!(results.len(), 3);
    assert!(results[0].score > 0.8);
    assert!(results[1].score > 0.8);
    assert!(results[2].score > 0.8);
}

#[tokio::test]
async fn test_unicode_documents() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add documents with various Unicode characters
    rag_system.add_document("Hello 世界", None).await.unwrap();
    rag_system.add_document("Привет мир", None).await.unwrap();
    rag_system.add_document("مرحبا العالم", None).await.unwrap();
    rag_system
        .add_document("🚀 Emoji test 🎉", None)
        .await
        .unwrap();

    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 4);

    // Search should work with Unicode
    let results = rag_system.search("世界", 2).await.unwrap();
    assert!(results.len() > 0);
}

#[tokio::test]
async fn test_very_long_document() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Create a very long document
    let long_text = "Lorem ipsum dolor sit amet. ".repeat(1000);

    let _id = rag_system.add_document(&long_text, None).await.unwrap();

    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 1);

    let results = rag_system.search("Lorem ipsum", 1).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].text.len(), long_text.len());
}

#[tokio::test]
async fn test_empty_document() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add empty document
    let _id = rag_system.add_document("", None).await.unwrap();

    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 1);

    let results = rag_system.search("anything", 1).await.unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn test_special_characters_in_document() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add documents with special characters
    rag_system
        .add_document("Test with\nnewlines\nand\ttabs", None)
        .await
        .unwrap();
    rag_system
        .add_document("Test with \"quotes\" and 'apostrophes'", None)
        .await
        .unwrap();
    rag_system
        .add_document("Test with symbols: @#$%^&*()", None)
        .await
        .unwrap();

    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 3);

    let results = rag_system.search("Test", 3).await.unwrap();
    assert_eq!(results.len(), 3);
}

#[tokio::test]
async fn test_metadata_with_complex_types() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add document with complex metadata
    let mut metadata = HashMap::new();
    metadata.insert("string".to_string(), serde_json::json!("value"));
    metadata.insert("number".to_string(), serde_json::json!(42));
    metadata.insert("float".to_string(), serde_json::json!(3.14));
    metadata.insert("boolean".to_string(), serde_json::json!(true));
    metadata.insert("array".to_string(), serde_json::json!([1, 2, 3]));
    metadata.insert("object".to_string(), serde_json::json!({"key": "value"}));

    let _id = rag_system
        .add_document("Document with complex metadata", Some(metadata))
        .await
        .unwrap();

    let results = rag_system.search("metadata", 1).await.unwrap();
    assert_eq!(results.len(), 1);

    let meta = results[0].metadata.as_ref().unwrap();
    assert_eq!(meta.get("string").and_then(|v| v.as_str()), Some("value"));
    assert_eq!(meta.get("number").and_then(|v| v.as_i64()), Some(42));
    assert_eq!(meta.get("boolean").and_then(|v| v.as_bool()), Some(true));
}

#[tokio::test]
async fn test_concurrent_operations() {
    use tokio::task;

    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system =
        std::sync::Arc::new(RAGSystem::new(Box::new(embeddings), Box::new(vector_store)));

    // Spawn multiple tasks adding documents concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let rag = rag_system.clone();
        let handle = task::spawn(async move {
            rag.add_document(&format!("Concurrent document {}", i), None)
                .await
                .unwrap();
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 10);
}

#[tokio::test]
async fn test_search_score_ordering() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add documents with varying similarity
    rag_system
        .add_document("apple banana cherry", None)
        .await
        .unwrap();
    rag_system.add_document("apple banana", None).await.unwrap();
    rag_system.add_document("apple", None).await.unwrap();
    rag_system
        .add_document("completely different text", None)
        .await
        .unwrap();

    let results = rag_system.search("apple banana", 4).await.unwrap();

    // Results should be ordered by score (descending)
    for i in 0..results.len() - 1 {
        assert!(
            results[i].score >= results[i + 1].score,
            "Results not properly ordered by score"
        );
    }
}

#[tokio::test]
async fn test_delete_nonexistent_document() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Delete non-existent document (should not error)
    let result = rag_system.delete_document("nonexistent-id").await;
    assert!(result.is_ok());

    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_multiple_deletes_same_document() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    let id = rag_system
        .add_document("Test document", None)
        .await
        .unwrap();

    // Delete once
    rag_system.delete_document(&id).await.unwrap();
    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 0);

    // Delete again (should not error)
    let result = rag_system.delete_document(&id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_clear_empty_store() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Clear empty store (should not error)
    let result = rag_system.clear().await;
    assert!(result.is_ok());

    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_multiple_clears() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add documents
    rag_system.add_document("Doc 1", None).await.unwrap();
    rag_system.add_document("Doc 2", None).await.unwrap();

    // Clear multiple times
    rag_system.clear().await.unwrap();
    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 0);

    rag_system.clear().await.unwrap();
    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_search_with_zero_limit() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    rag_system
        .add_document("Test document", None)
        .await
        .unwrap();

    // Search with limit 0
    let results = rag_system.search("Test", 0).await.unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_dimension_consistency() {
    let embeddings = FixedDimensionEmbeddings::new(256);
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add multiple documents
    for i in 0..5 {
        rag_system
            .add_document(&format!("Document {}", i), None)
            .await
            .unwrap();
    }

    // Search should work correctly
    let results = rag_system.search("Document", 5).await.unwrap();
    assert_eq!(results.len(), 5);
}

#[tokio::test]
async fn test_large_batch_operations() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add 100 documents
    let mut ids = Vec::new();
    for i in 0..100 {
        let id = rag_system
            .add_document(&format!("Document number {}", i), None)
            .await
            .unwrap();
        ids.push(id);
    }

    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 100);

    // Search should return results efficiently
    let results = rag_system.search("Document", 20).await.unwrap();
    assert_eq!(results.len(), 20);

    // Delete half the documents
    for id in ids.iter().take(50) {
        rag_system.delete_document(id).await.unwrap();
    }

    let count = rag_system.count().await.unwrap();
    assert_eq!(count, 50);
}

#[tokio::test]
async fn test_timestamp_metadata() {
    let embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    // Add document (timestamp should be added automatically)
    let _id = rag_system.add_document("Test", None).await.unwrap();

    let results = rag_system.search("Test", 1).await.unwrap();
    assert_eq!(results.len(), 1);

    // Check that timestamp exists in metadata
    let meta = results[0].metadata.as_ref().unwrap();
    assert!(meta.contains_key("timestamp"));
}

#[tokio::test]
async fn test_replicate_same_id_behavior() {
    let _embeddings = MockEmbeddings;
    let vector_store = InMemoryVectorStore::new();

    // Directly test vector store behavior
    vector_store.initialize(128).await.unwrap();

    let embedding = vec![0.1; 128];
    let metadata = HashMap::new();

    // Add document with specific ID
    vector_store
        .add("test-id", embedding.clone(), "First text", metadata.clone())
        .await
        .unwrap();
    let count = vector_store.count().await.unwrap();
    assert_eq!(count, 1);

    // Add another document with same ID (should replace)
    vector_store
        .add(
            "test-id",
            embedding.clone(),
            "Second text",
            metadata.clone(),
        )
        .await
        .unwrap();
    let count = vector_store.count().await.unwrap();
    assert_eq!(count, 1);

    // Search should return the second text
    let results = vector_store.search(embedding, 1).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].text, "Second text");
}
