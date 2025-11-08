# Vector Stores

Vector stores are responsible for storing and retrieving document embeddings. Helios Engine supports two vector stores out of the box: an in-memory store and a Qdrant store.

## In-Memory Vector Store

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

## Qdrant Vector Store

A production-ready vector store using the [Qdrant](https://qdrant.tech/) vector database.

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

### Setting up Qdrant

You can run Qdrant using Docker:

```bash
docker run -p 6333:6333 qdrant/qdrant
```
