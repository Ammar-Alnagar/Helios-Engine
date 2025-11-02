//! # Example: Local Model Streaming
//!
//! This example demonstrates how to use the streaming capabilities of the Helios Engine
//! with a local model. The response from the model is streamed token by token,
//! providing a real-time experience.
//!
//! Note: This example requires the `local` feature to be enabled.
//! Run with: cargo run --example local_streaming --features local

#[cfg(not(feature = "local"))]
fn main() {
    eprintln!("❌ This example requires the 'local' feature to be enabled.");
    eprintln!("Run with: cargo run --example local_streaming --features local");
    std::process::exit(1);
}

#[cfg(feature = "local")]
use helios_engine::config::LocalConfig;
#[cfg(feature = "local")]
use helios_engine::{ChatMessage, LLMClient};
#[cfg(feature = "local")]
use std::io::{self, Write};

#[cfg(feature = "local")]
#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Helios Engine - Local Model Streaming Example");
    println!("=================================================\n");

    // Configure the local model to use.
    let local_config = LocalConfig {
        huggingface_repo: "unsloth/Qwen2.5-0.5B-Instruct-GGUF".to_string(),
        model_file: "Qwen2.5-0.5B-Instruct-Q4_K_M.gguf".to_string(),
        context_size: 2048,
        temperature: 0.7,
        max_tokens: 512,
    };

    println!("📥 Loading local model...");
    println!("   Repository: {}", local_config.huggingface_repo);
    println!("   Model: {}\n", local_config.model_file);

    // Create a new LLM client with the local model configuration.
    let client = LLMClient::new(helios_engine::llm::LLMProviderType::Local(local_config)).await?;

    println!("✓ Model loaded successfully!\n");

    // --- Example 1: Simple streaming response ---
    println!("Example 1: Simple Streaming Response");
    println!("======================================\n");

    let messages = vec![
        ChatMessage::system("You are a helpful coding assistant."),
        ChatMessage::user("Write a short explanation of what Rust is."),
    ];

    print!("Assistant: ");
    io::stdout().flush()?;

    // Stream the response from the model, printing each chunk as it arrives.
    let _response = client
        .chat_stream(messages, None, None, None, None, |chunk| {
            print!("{}", chunk);
            io::stdout().flush().unwrap();
        })
        .await?;

    println!("\n");

    // --- Example 2: Interactive streaming ---
    println!("Example 2: Interactive Streaming");
    println!("==================================\n");

    let questions = vec![
        "What are the main benefits of Rust?",
        "Give me a simple code example.",
    ];

    let mut session = helios_engine::ChatSession::new()
        .with_system_prompt("You are a helpful programming assistant.");

    for question in questions {
        println!("User: {}", question);
        session.add_user_message(question);

        print!("Assistant: ");
        io::stdout().flush()?;

        // Stream the response, maintaining the conversation context.
        let response = client
            .chat_stream(session.get_messages(), None, None, None, None, |chunk| {
                print!("{}", chunk);
                io::stdout().flush().unwrap();
            })
            .await?;

        session.add_assistant_message(&response.content);
        println!("\n");
    }

    println!("✅ Local model streaming completed successfully!");
    println!("\n💡 Features:");
    println!("  • Token-by-token streaming for local models");
    println!("  • Real-time response display (no more instant full responses)");
    println!("  • Same streaming API for both local and remote models");
    println!("  • Improved user experience with progressive output");

    Ok(())
}
