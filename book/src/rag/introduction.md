# Introduction to Retrieval-Augmented Generation (RAG)

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

## Usage with Agents

The simplest way to use RAG is through the `RAGTool` with an agent.

### In-Memory RAG

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

### Qdrant RAG

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
