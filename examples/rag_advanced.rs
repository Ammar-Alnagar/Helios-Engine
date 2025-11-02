//! # Example: Advanced RAG Usage
//!
//! This example demonstrates advanced RAG features including:
//! - Custom metadata
//! - Document management (add, search, delete, count)
//! - Direct RAG system usage (without agent)
//! - Batch operations
//!
//! ## Prerequisites
//!
//! - OpenAI API Key: `export OPENAI_API_KEY=your-key`

use helios_engine::{InMemoryVectorStore, OpenAIEmbeddings, RAGSystem, SearchResult};
use std::collections::HashMap;

// Helper macro for creating HashMaps
macro_rules! hashmap {
    ($($key:expr => $val:expr),* $(,)?) => {{
        let mut map = HashMap::new();
        $(map.insert($key, $val);)*
        map
    }};
}

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Helios Engine - Advanced RAG Features");
    println!("========================================\n");

    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        println!("⚠ Warning: OPENAI_API_KEY not set. Using placeholder.");
        "your-api-key-here".to_string()
    });

    // Create RAG system directly (without agent)
    let embeddings = OpenAIEmbeddings::new("https://api.openai.com/v1/embeddings", api_key);
    let vector_store = InMemoryVectorStore::new();
    let rag_system = RAGSystem::new(Box::new(embeddings), Box::new(vector_store));

    println!("✓ RAG system created\n");

    // --- Example 1: Adding documents with metadata ---
    println!("Example 1: Documents with Metadata");
    println!("===================================\n");

    let documents = vec![
        (
            "Rust is a systems programming language focused on safety and performance.",
            hashmap! {
                "category" => "programming",
                "language" => "rust",
                "year" => "2010",
                "difficulty" => "intermediate",
            },
        ),
        (
            "Python is known for its simplicity and extensive library ecosystem.",
            hashmap! {
                "category" => "programming",
                "language" => "python",
                "year" => "1991",
                "difficulty" => "beginner",
            },
        ),
        (
            "Machine learning is a subset of AI that enables systems to learn from data.",
            hashmap! {
                "category" => "ai",
                "topic" => "machine-learning",
                "difficulty" => "advanced",
            },
        ),
        (
            "Docker is a platform for developing, shipping, and running applications in containers.",
            hashmap! {
                "category" => "devops",
                "tool" => "docker",
                "year" => "2013",
            },
        ),
    ];

    let mut doc_ids = Vec::new();
    for (text, meta) in documents.iter() {
        let metadata: HashMap<String, serde_json::Value> = meta
            .iter()
            .map(|(k, v)| (k.to_string(), serde_json::json!(v)))
            .collect();

        let id = rag_system.add_document(text, Some(metadata)).await?;
        println!(
            "Added document: {} (ID: {})",
            &text[..50.min(text.len())],
            id
        );
        doc_ids.push(id);
    }
    println!();

    // --- Example 2: Semantic search ---
    println!("Example 2: Semantic Search");
    println!("==========================\n");

    let queries = vec![
        ("programming language safety", 3),
        ("containerization technology", 2),
        ("artificial intelligence", 2),
    ];

    for (query, limit) in queries {
        println!("Query: '{}' (limit: {})", query, limit);
        let results = rag_system.search(query, limit).await?;
        print_results(&results);
        println!();
    }

    // --- Example 3: Document count ---
    println!("Example 3: Document Management");
    println!("===============================\n");

    let count = rag_system.count().await?;
    println!("Total documents: {}\n", count);

    // --- Example 4: Delete a document ---
    if let Some(first_id) = doc_ids.first() {
        println!("Deleting document: {}", first_id);
        rag_system.delete_document(first_id).await?;
        let new_count = rag_system.count().await?;
        println!("Documents after deletion: {}\n", new_count);
    }

    // --- Example 5: Search after deletion ---
    println!("Example 5: Search After Deletion");
    println!("=================================\n");

    let results = rag_system.search("programming languages", 5).await?;
    println!("Results for 'programming languages':");
    print_results(&results);
    println!();

    // --- Example 6: Clear all documents ---
    println!("Example 6: Clear All Documents");
    println!("===============================\n");

    rag_system.clear().await?;
    let final_count = rag_system.count().await?;
    println!("Documents after clear: {}\n", final_count);

    println!("✅ Example completed successfully!");
    println!("\n💡 Key Features Demonstrated:");
    println!("  • Direct RAG system usage (no agent required)");
    println!("  • Documents with custom metadata");
    println!("  • Semantic search with configurable limits");
    println!("  • Document management (add, delete, count, clear)");
    println!("  • Batch operations");

    println!("\n📝 Advanced Use Cases:");
    println!("  • Building custom RAG pipelines");
    println!("  • Document management systems");
    println!("  • Knowledge base applications");
    println!("  • Semantic search engines");

    Ok(())
}

fn print_results(results: &[SearchResult]) {
    if results.is_empty() {
        println!("  No results found");
        return;
    }

    for (i, result) in results.iter().enumerate() {
        let preview = if result.text.len() > 80 {
            format!("{}...", &result.text[..80])
        } else {
            result.text.clone()
        };
        println!("  {}. [Score: {:.4}] {}", i + 1, result.score, preview);
        if let Some(metadata) = &result.metadata {
            let meta_str: Vec<String> = metadata
                .iter()
                .filter(|(k, _)| k.as_str() != "timestamp")
                .map(|(k, v)| format!("{}={}", k, v.as_str().unwrap_or("?")))
                .collect();
            if !meta_str.is_empty() {
                println!("     Metadata: {}", meta_str.join(", "));
            }
        }
    }
}
