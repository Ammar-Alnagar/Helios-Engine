# Embedding Providers

Embedding providers are responsible for generating vector embeddings from text. Helios Engine supports OpenAI's embedding API out of the box.

## OpenAI Embeddings

Uses OpenAI's embedding API (e.g., `text-embedding-ada-002` or `text-embedding-3-small`).

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
- 1536 dimensions (for `text-embedding-ada-002` and `text-embedding-3-small`)
- Excellent for semantic search
- Requires an API key and an internet connection
