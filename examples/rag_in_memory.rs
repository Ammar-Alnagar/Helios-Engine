//! # Example: Agent with In-Memory RAG
//!
//! This example demonstrates how to create an agent with an in-memory RAG system.
//! The in-memory vector store is perfect for development, testing, or when you
//! don't need persistence across restarts.
//!
//! ## Prerequisites
//!
//! - **OpenAI API Key**: You need an OpenAI API key for generating embeddings.
//!   Set it as an environment variable:
//!   ```sh
//!   export OPENAI_API_KEY=your-key
//!   ```

use helios_engine::{Agent, Config, RAGTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Helios Engine - Agent with In-Memory RAG Example");
    println!("===================================================\n");

    // Check for the required OpenAI API key
    let embedding_api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        println!("⚠ Warning: OPENAI_API_KEY not set. Using placeholder.");
        "your-api-key-here".to_string()
    });

    // Load configuration
    let config = Config::from_file("config.toml").unwrap_or_else(|_| {
        println!("⚠ No config.toml found, using default configuration");
        Config::new_default()
    });

    // Create a new RAG tool with in-memory vector store
    let rag_tool =
        RAGTool::new_in_memory("https://api.openai.com/v1/embeddings", embedding_api_key);

    // Create an agent with RAG capabilities
    let mut agent = Agent::builder("KnowledgeAgent")
        .config(config)
        .system_prompt(
            "You are a helpful assistant with access to an in-memory RAG (Retrieval-Augmented Generation) system. \
             You can store documents and retrieve relevant information to answer questions. \
             When answering questions, first search for relevant documents, then provide informed answers based on the retrieved context."
        )
        .tool(Box::new(rag_tool))
        .max_iterations(10)
        .build()
        .await?;

    println!("✓ Agent created with in-memory RAG capabilities\n");

    // --- Example 1: Add knowledge about programming languages ---
    println!("Example 1: Building a Knowledge Base");
    println!("=====================================\n");

    let documents = [
        "Rust is a systems programming language that runs blazingly fast, prevents segfaults, \
         and guarantees thread safety. It was created by Mozilla Research and first released in 2010.",
        "Python is a high-level, interpreted programming language known for its clear syntax \
         and readability. It was created by Guido van Rossum and first released in 1991.",
        "JavaScript is a programming language commonly used for web development. It allows \
         developers to create interactive web pages and runs in web browsers. It was created in 1995.",
        "Go is a statically typed, compiled programming language designed at Google. It is \
         syntactically similar to C, but with memory safety and garbage collection.",
        "TypeScript is a strongly typed programming language that builds on JavaScript. It was \
         developed and maintained by Microsoft and first released in 2012.",
    ];

    for (i, doc) in documents.iter().enumerate() {
        println!("Adding document {}...", i + 1);
        let response = agent
            .chat(&format!("Store this information: {}", doc))
            .await?;
        println!("Agent: {}\n", response);
    }

    // --- Example 2: Semantic search with different queries ---
    println!("\nExample 2: Semantic Search");
    println!("==========================\n");

    let queries = vec![
        "What programming language prevents segfaults?",
        "Tell me about the language created by Guido van Rossum",
        "Which language is used for web development in browsers?",
        "What language was developed by Google?",
    ];

    for query in queries {
        println!("Query: {}", query);
        let response = agent.chat(query).await?;
        println!("Agent: {}\n", response);
    }

    // --- Example 3: Check document count ---
    println!("\nExample 3: Document Count");
    println!("=========================\n");

    let response = agent.chat("How many documents are stored?").await?;
    println!("Agent: {}\n", response);

    // --- Example 4: Complex query requiring multiple documents ---
    println!("\nExample 4: Complex Query");
    println!("========================\n");

    let response = agent
        .chat("Compare the programming languages that were created in the 1990s")
        .await?;
    println!("Agent: {}\n", response);

    println!("\n✅ Example completed successfully!");
    println!("\n💡 Key Features Demonstrated:");
    println!("  • In-memory vector storage (no external dependencies)");
    println!("  • Document embedding with OpenAI embeddings");
    println!("  • Semantic search with cosine similarity");
    println!("  • RAG workflow for context-aware answers");
    println!("  • Fast performance for development and testing");

    println!("\n📝 Use Cases for In-Memory RAG:");
    println!("  • Development and testing");
    println!("  • Short-lived sessions");
    println!("  • When persistence is not required");
    println!("  • Rapid prototyping");

    println!("\n🔧 Advantages:");
    println!("  • No external dependencies (no database needed)");
    println!("  • Fast setup and execution");
    println!("  • Simple deployment");
    println!("  • Perfect for demos and examples");

    Ok(())
}
