//! # Example: Comparing In-Memory and Qdrant RAG
//!
//! This example demonstrates both in-memory and Qdrant RAG implementations,
//! showing when to use each one and how to switch between them.
//!
//! ## Prerequisites
//!
//! 1. **Qdrant** (optional, only for Qdrant backend): Start with Docker:
//!    ```sh
//!    docker run -p 6333:6333 qdrant/qdrant
//!    ```
//! 2. **OpenAI API Key**: Required for embeddings:
//!    ```sh
//!    export OPENAI_API_KEY=your-key
//!    ```

use helios_engine::{Agent, Config, RAGTool};

async fn demonstrate_in_memory() -> helios_engine::Result<()> {
    println!("=== IN-MEMORY RAG DEMONSTRATION ===\n");

    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        println!("⚠ Warning: OPENAI_API_KEY not set. Using placeholder.");
        "your-api-key-here".to_string()
    });

    let config = Config::from_file("config.toml").unwrap_or_else(|_| Config::new_default());

    // Create in-memory RAG tool
    let rag_tool = RAGTool::new_in_memory("https://api.openai.com/v1/embeddings", &api_key);

    let mut agent = Agent::builder("InMemoryAgent")
        .config(config)
        .system_prompt("You are a helpful assistant with in-memory RAG capabilities.")
        .tool(Box::new(rag_tool))
        .max_iterations(8)
        .build()
        .await?;

    println!("✓ In-memory agent created\n");

    // Add some documents
    println!("Adding documents...");
    agent.chat("Store: The capital of France is Paris.").await?;
    agent
        .chat("Store: The capital of Germany is Berlin.")
        .await?;
    agent.chat("Store: The capital of Italy is Rome.").await?;
    println!("✓ Documents added\n");

    // Search
    println!("Searching...");
    let response = agent.chat("What is the capital of Germany?").await?;
    println!("Agent: {}\n", response);

    println!("Advantages of in-memory:");
    println!("  ✓ No external dependencies");
    println!("  ✓ Fast and simple");
    println!("  ✓ Perfect for development");
    println!("  ✗ No persistence (data lost on restart)");
    println!("  ✗ Limited scalability\n");

    Ok(())
}

async fn demonstrate_qdrant() -> helios_engine::Result<()> {
    println!("=== QDRANT RAG DEMONSTRATION ===\n");

    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        println!("⚠ Warning: OPENAI_API_KEY not set. Using placeholder.");
        "your-api-key-here".to_string()
    });

    let config = Config::from_file("config.toml").unwrap_or_else(|_| Config::new_default());

    // Create Qdrant RAG tool
    let rag_tool = RAGTool::new_qdrant(
        "http://localhost:6333",
        "comparison_demo",
        "https://api.openai.com/v1/embeddings",
        &api_key,
    );

    let mut agent = Agent::builder("QdrantAgent")
        .config(config)
        .system_prompt("You are a helpful assistant with Qdrant RAG capabilities.")
        .tool(Box::new(rag_tool))
        .max_iterations(8)
        .build()
        .await?;

    println!("✓ Qdrant agent created\n");

    // Clear any existing data
    println!("Clearing existing data...");
    agent.chat("Clear all documents").await?;
    println!("✓ Cleared\n");

    // Add some documents
    println!("Adding documents...");
    agent
        .chat("Store: The Eiffel Tower is located in Paris, France.")
        .await?;
    agent
        .chat("Store: The Colosseum is located in Rome, Italy.")
        .await?;
    agent
        .chat("Store: The Brandenburg Gate is located in Berlin, Germany.")
        .await?;
    println!("✓ Documents added\n");

    // Search
    println!("Searching...");
    let response = agent.chat("What famous landmark is in Berlin?").await?;
    println!("Agent: {}\n", response);

    println!("Advantages of Qdrant:");
    println!("  ✓ Persistent storage");
    println!("  ✓ Highly scalable");
    println!("  ✓ Production-ready");
    println!("  ✓ Advanced features (filtering, etc.)");
    println!("  ✗ Requires external service");
    println!("  ✗ More complex setup\n");

    Ok(())
}

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Helios Engine - RAG Backend Comparison");
    println!("=========================================\n");

    // Demonstrate in-memory RAG
    match demonstrate_in_memory().await {
        Ok(_) => println!("✅ In-memory demonstration completed\n"),
        Err(e) => println!("❌ In-memory demonstration failed: {}\n", e),
    }

    println!("---\n");

    // Demonstrate Qdrant RAG
    match demonstrate_qdrant().await {
        Ok(_) => println!("✅ Qdrant demonstration completed\n"),
        Err(e) => {
            println!("❌ Qdrant demonstration failed: {}", e);
            println!("   (Make sure Qdrant is running: docker run -p 6333:6333 qdrant/qdrant)\n");
        }
    }

    println!("\n📊 Comparison Summary:");
    println!("┌──────────────────┬────────────────┬──────────────┐");
    println!("│ Feature          │ In-Memory      │ Qdrant       │");
    println!("├──────────────────┼────────────────┼──────────────┤");
    println!("│ Setup            │ Easy           │ Moderate     │");
    println!("│ Dependencies     │ None           │ Qdrant       │");
    println!("│ Persistence      │ No             │ Yes          │");
    println!("│ Scalability      │ Limited        │ High         │");
    println!("│ Performance      │ Fast (in-mem)  │ Fast         │");
    println!("│ Best for         │ Dev/Testing    │ Production   │");
    println!("└──────────────────┴────────────────┴──────────────┘");

    println!("\n💡 Recommendations:");
    println!("  • Use in-memory for: development, testing, demos");
    println!("  • Use Qdrant for: production, large datasets, persistence");

    Ok(())
}
