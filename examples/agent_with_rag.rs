//! # Example: Agent with RAG (Retrieval-Augmented Generation)
//!
//! This example demonstrates how to create an agent with a `QdrantRAGTool` to
//! perform Retrieval-Augmented Generation. The agent can store documents in a
//! Qdrant vector database, perform semantic search, and use the retrieved
//! information to answer questions.
//!
//! ## Prerequisites
//!
//! 1.  **Qdrant**: You need a running Qdrant instance. You can start one with Docker:
//!     ```sh
//!     docker run -p 6333:6333 qdrant/qdrant
//!     ```
//! 2.  **OpenAI API Key**: You need an OpenAI API key for generating embeddings.
//!     Set it as an environment variable:
//!     ```sh
//!     export OPENAI_API_KEY=your-key
//!     ```

use helios_engine::{Agent, Config, QdrantRAGTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Helios Engine - Agent with RAG Example");
    println!("==========================================\n");

    // Check for the required OpenAI API key.
    let embedding_api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        println!("⚠ Warning: OPENAI_API_KEY not set. Using placeholder.");
        "your-api-key-here".to_string()
    });

    // Load configuration from `config.toml` or use default.
    let config = Config::from_file("config.toml").unwrap_or_else(|_| {
        println!("⚠ No config.toml found, using default configuration");
        Config::new_default()
    });

    // Create a new `QdrantRAGTool`.
    let rag_tool = QdrantRAGTool::new(
        "http://localhost:6333",                // Qdrant URL
        "helios_knowledge",                     // Collection name
        "https://api.openai.com/v1/embeddings", // Embedding API
        embedding_api_key,                      // API key
    );

    // Create an agent named "KnowledgeAgent" and equip it with the RAG tool.
    let mut agent = Agent::builder("KnowledgeAgent")
        .config(config)
        .system_prompt(
            "You are a helpful assistant with access to a RAG (Retrieval-Augmented Generation) system. \
             You can store documents and retrieve relevant information to answer questions. \
             When answering questions, first search for relevant documents, then provide informed answers based on the retrieved context."
        )
        .tool(Box::new(rag_tool))
        .max_iterations(10)
        .build()
        .await?;

    println!("✓ Agent created with RAG capabilities\n");

    // --- Example 1: Add knowledge to the database ---
    println!("Example 1: Adding Documents to Knowledge Base");
    println!("==============================================\n");

    let response = agent
        .chat(
            "Store this information: Rust is a systems programming language that runs blazingly fast, \
             prevents segfaults, and guarantees thread safety. It was created by Mozilla Research."
        )
        .await?;
    println!("Agent: {}\n", response);

    let response = agent
        .chat(
            "Store this: Python is a high-level, interpreted programming language known for its \
             clear syntax and readability. It was created by Guido van Rossum in 1991.",
        )
        .await?;
    println!("Agent: {}\n", response);

    let response = agent
        .chat(
            "Store this: JavaScript is a programming language commonly used for web development. \
             It allows developers to create interactive web pages and runs in web browsers.",
        )
        .await?;
    println!("Agent: {}\n", response);

    // --- Example 2: Semantic search - ask questions ---
    println!("\nExample 2: Semantic Search and Q&A");
    println!("===================================\n");

    let response = agent
        .chat("What programming language is known for preventing segfaults?")
        .await?;
    println!("Agent: {}\n", response);

    let response = agent
        .chat("Tell me about the programming language created in 1991")
        .await?;
    println!("Agent: {}\n", response);

    // --- Example 3: Multi-document retrieval ---
    println!("\nExample 3: Multi-Document Retrieval");
    println!("====================================\n");

    let response = agent
        .chat("Search for information about programming languages and summarize what you find")
        .await?;
    println!("Agent: {}\n", response);

    // --- Example 4: Adding documents with metadata ---
    println!("\nExample 4: Documents with Metadata");
    println!("===================================\n");

    let response = agent
        .chat(
            "Store this with metadata: \
             The Helios Engine is a Rust framework for building LLM agents. \
             Metadata: category=framework, language=rust, year=2025",
        )
        .await?;
    println!("Agent: {}\n", response);

    println!("\n✅ Example completed successfully!");
    println!("\n💡 Key Features Demonstrated:");
    println!("  • Document embedding with OpenAI embeddings");
    println!("  • Vector storage in Qdrant database");
    println!("  • Semantic search with cosine similarity");
    println!("  • RAG workflow for context-aware answers");
    println!("  • Metadata support for document organization");

    println!("\n📝 RAG Use Cases:");
    println!("  • Question answering over custom knowledge bases");
    println!("  • Document search and retrieval");
    println!("  • Building chatbots with domain-specific knowledge");
    println!("  • Information extraction from large document sets");

    println!("\n🔧 Setup Instructions:");
    println!("  1. Start Qdrant: docker run -p 6333:6333 qdrant/qdrant");
    println!("  2. Set API key: export OPENAI_API_KEY=your-key");
    println!("  3. Run example: cargo run --example agent_with_rag");

    Ok(())
}
