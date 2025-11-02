# RAG (Retrieval-Augmented Generation) Guide

## Overview

Helios Engine provides a powerful and flexible RAG (Retrieval-Augmented Generation) system that allows agents to store and retrieve documents using semantic search. The system supports multiple backends and embedding providers, making it suitable for both development and production use.

## Architecture

The RAG system consists of three main components:

1. **Embedding Provider**: Generates vector embeddings from text
2. **Vector Store**: Stores and retrieves document embeddings
3. **RAG System**: Coordinates embedding and storage operations

```
┌─────────────────┐
│   RAG System    │
├─────────────────┤
│  • add_document │
│  • search       │
│  • delete       │
│  • clear        │
│  • count        │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
┌───▼──────┐  ┌──▼───────────┐
│Embedding │  │Vector Store  │
│Provider  │  │              │
├──────────┤  ├──────────────┤
│ OpenAI   │  │ In-Memory    │
│ (custom) │  │ Qdrant       │
└──────────┘  └──────────────┘
```

## Components

### Embedding Providers

#### OpenAI Embeddings

Uses OpenAI's embedding API (text-embedding-ada-002 or text-embedding-3-small/large).

```rust
use helios_engine::OpenAIEmbeddings;

let embeddings = OpenAIEmbeddings::new(
    "https://api.openai.com/v1/embeddings",
    std::env::var("OPENAI_API_KEY").unwrap()
);

// Or with a specific model
let embeddings = OpenAIEmbeddings::with_model(
    "https://api.openai.com/v1/embeddings",
    std::env::var("OPENAI_API_KEY").unwrap(),
    "text-embedding-3-small"
);
```

**Features:**
- High-quality embeddings
- 1536 dimensions (ada-002, 3-small) or 3072 (3-large)
- Excellent for semantic search
- Requires API key and internet connection

### Vector Stores

#### In-Memory Vector Store

A fast, lightweight vector store that keeps all data in memory.

```rust
use helios_engine::InMemoryVectorStore;

let vector_store = InMemoryVectorStore::new();
```

**Advantages:**
- ✓ No external dependencies
- ✓ Fast performance
- ✓ Simple setup
- ✓ Perfect for development and testing

**Disadvantages:**
- ✗ No persistence (data lost on restart)
- ✗ Limited by available memory
- ✗ Not suitable for large datasets

**Use Cases:**
- Development and testing
- Demos and examples
- Short-lived sessions
- Prototyping

#### Qdrant Vector Store

A production-ready vector store using Qdrant database.

```rust
use helios_engine::QdrantVectorStore;

let vector_store = QdrantVectorStore::new(
    "http://localhost:6333",
    "my_collection"
);
```

**Advantages:**
- ✓ Persistent storage
- ✓ Highly scalable
- ✓ Production-ready
- ✓ Advanced features (filtering, etc.)

**Disadvantages:**
- ✗ Requires Qdrant service
- ✗ More complex setup

**Use Cases:**
- Production applications
- Large datasets
- Multi-user systems
- When persistence is required

**Setup Qdrant:**
```bash
docker run -p 6333:6333 qdrant/qdrant
```

## Usage

### Using RAG with Agents

The simplest way to use RAG is through the `RAGTool` with an agent.

#### In-Memory RAG

```rust
use helios_engine::{Agent, Config, RAGTool};

let config = Config::from_file("config.toml").unwrap_or_default();
let rag_tool = RAGTool::new_in_memory(
    "https://api.openai.com/v1/embeddings",
    std::env::var("OPENAI_API_KEY").unwrap()
);

let mut agent = Agent::builder("KnowledgeAgent")
    .config(config)
    .tool(Box::new(rag_tool))
    .build()
    .await?;

// Add documents
agent.chat("Store this: Rust is a systems programming language.").await?;

// Search
let response = agent.chat("What do you know about Rust?").await?;
```

#### Qdrant RAG

```rust
let config = Config::from_file("config.toml").unwrap_or_default();
let rag_tool = RAGTool::new_qdrant(
    "http://localhost:6333",
    "my_collection",
    "https://api.openai.com/v1/embeddings",
    std::env::var("OPENAI_API_KEY").unwrap()
);

let mut agent = Agent::builder("KnowledgeAgent")
    .config(config)
    .tool(Box::new(rag_tool))
    .build()
    .await?;
```

### Direct RAG System Usage

For more control, use the RAG system directly without an agent.

```rust
use helios_engine::{
    RAGSystem, OpenAIEmbeddings, InMemoryVectorStore
};

// Create components
let embeddings = OpenAIEmbeddings::new(
    "https://api.openai.com/v1/embeddings",
    std::env::var("OPENAI_API_KEY").unwrap()
);
let vector_store = InMemoryVectorStore::new();

// Create RAG system
let rag_system = RAGSystem::new(
    Box::new(embeddings),
    Box::new(vector_store)
);

// Add documents
let doc_id = rag_system.add_document(
    "Rust is a systems programming language.",
    None // Optional metadata
).await?;

// Search
let results = rag_system.search("systems programming", 5).await?;
for result in results {
    println!("Score: {:.4} - {}", result.score, result.text);
}

// Delete
rag_system.delete_document(&doc_id).await?;

// Count
let count = rag_system.count().await?;

// Clear all
rag_system.clear().await?;
```

### Adding Metadata

You can attach metadata to documents for better organization:

```rust
use std::collections::HashMap;

let mut metadata = HashMap::new();
metadata.insert("category".to_string(), serde_json::json!("programming"));
metadata.insert("language".to_string(), serde_json::json!("rust"));
metadata.insert("year".to_string(), serde_json::json!("2010"));

let doc_id = rag_system.add_document(
    "Rust is a systems programming language.",
    Some(metadata)
).await?;
```

### RAG Tool Operations

When using the RAG tool with an agent, the following operations are available:

#### Add Document
```rust
agent.chat("Store this information: Your document text here").await?;
```

#### Search
```rust
agent.chat("Search for information about Rust programming").await?;
```

#### Delete Document
```rust
// First get the document ID from a previous operation
agent.chat("Delete document with ID: abc-123").await?;
```

#### Count Documents
```rust
agent.chat("How many documents are stored?").await?;
```

#### Clear All Documents
```rust
agent.chat("Clear all documents").await?;
```

## Advanced Features

### Custom Embedding Providers

You can implement your own embedding provider by implementing the `EmbeddingProvider` trait:

```rust
use helios_engine::{EmbeddingProvider, Result};
use async_trait::async_trait;

struct CustomEmbeddings {
    // Your fields
}

#[async_trait]
impl EmbeddingProvider for CustomEmbeddings {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Your embedding logic
        todo!()
    }

    fn dimension(&self) -> usize {
        // Return embedding dimension
        1536
    }
}
```

### Custom Vector Stores

Similarly, you can implement custom vector stores:

```rust
use helios_engine::{VectorStore, SearchResult, Result};
use async_trait::async_trait;
use std::collections::HashMap;

struct CustomVectorStore {
    // Your fields
}

#[async_trait]
impl VectorStore for CustomVectorStore {
    async fn initialize(&self, dimension: usize) -> Result<()> {
        // Initialize your store
        Ok(())
    }

    async fn add(
        &self,
        id: &str,
        embedding: Vec<f32>,
        text: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        // Add document
        todo!()
    }

    async fn search(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>> {
        // Search logic
        todo!()
    }

    async fn delete(&self, id: &str) -> Result<()> {
        // Delete logic
        todo!()
    }

    async fn clear(&self) -> Result<()> {
        // Clear logic
        todo!()
    }

    async fn count(&self) -> Result<usize> {
        // Count logic
        todo!()
    }
}
```

## Performance Considerations

### In-Memory Store
- **Speed**: Very fast, all operations in memory
- **Memory**: O(n) where n is number of documents
- **Scalability**: Limited by available RAM

### Qdrant Store
- **Speed**: Fast, with network overhead
- **Memory**: Low (data stored externally)
- **Scalability**: Very high (distributed support)

## Best Practices

1. **Choose the Right Backend**
   - Use in-memory for development and testing
   - Use Qdrant for production and persistence

2. **Chunk Large Documents**
   - Break large documents into smaller chunks
   - Add metadata to track chunk relationships

3. **Use Metadata Wisely**
   - Add relevant metadata for filtering
   - Include timestamps for temporal queries

4. **Optimize Search Limits**
   - Start with limit=5 for most queries
   - Increase only if needed (trade-off with performance)

5. **Handle Errors Gracefully**
   - Check for API key availability
   - Handle network failures for Qdrant
   - Validate embedding dimensions

## Examples

See the `examples/` directory for complete working examples:

- `examples/rag_in_memory.rs` - In-memory RAG with agent
- `examples/rag_qdrant_comparison.rs` - Compare backends
- `examples/rag_advanced.rs` - Advanced features and direct API usage
- `examples/agent_with_rag.rs` - Original Qdrant example

Run examples with:
```bash
cargo run --example rag_in_memory
cargo run --example rag_qdrant_comparison
cargo run --example rag_advanced
```

## Troubleshooting

### OpenAI API Errors
```
Error: Embedding API failed: 401 Unauthorized
```
**Solution**: Check that `OPENAI_API_KEY` is set correctly.

### Qdrant Connection Errors
```
Error: Failed to create collection: Connection refused
```
**Solution**: Ensure Qdrant is running:
```bash
docker run -p 6333:6333 qdrant/qdrant
```

### Dimension Mismatch
```
Error: Vector dimension mismatch
```
**Solution**: Don't mix embeddings from different models in the same collection.

## Comparison with QdrantRAGTool

Helios Engine provides two RAG implementations:

### Legacy: `QdrantRAGTool`
- Single backend (Qdrant only)
- Tightly coupled implementation
- Simple API

### New: `RAGTool` + `RAGSystem`
- Multiple backends (in-memory, Qdrant)
- Modular architecture
- Extensible (custom backends and embeddings)
- Same simple API for agents
- Direct API available for advanced use

**Migration Path:**
```rust
// Old
let rag_tool = QdrantRAGTool::new(
    "http://localhost:6333",
    "collection",
    "https://api.openai.com/v1/embeddings",
    api_key
);

// New (equivalent)
let rag_tool = RAGTool::new_qdrant(
    "http://localhost:6333",
    "collection",
    "https://api.openai.com/v1/embeddings",
    api_key
);
```

Both tools continue to be supported.
