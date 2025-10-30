use helios_engine::{ChatMessage, LLMClient};
use helios_engine::config::LocalConfig;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    eprintln!("Starting test...");
    
    let local_config = LocalConfig {
        huggingface_repo: "unsloth/Qwen3-4B-Instruct-2507-GGUF".to_string(),
        model_file: "Qwen3-4B-Instruct-2507-Q4_K_M.gguf".to_string(),
        context_size: 8192,
        temperature: 0.7,
        max_tokens: 512,
    };

    eprintln!("Loading model...");
    let client = LLMClient::new(
        helios_engine::llm::LLMProviderType::Local(local_config)
    ).await?;

    eprintln!("Model loaded, creating messages...");
    let messages = vec![
        ChatMessage::system("You are a helpful assistant."),
        ChatMessage::user("Tell me a short joke about programming."),
    ];

    eprintln!("Starting streaming...");
    print!("Response: ");
    io::stdout().flush()?;
    
    let response = client.chat_stream(messages, None, |chunk| {
        eprintln!("Got chunk: {:?}", chunk);
        print!("{}", chunk);
        io::stdout().flush().unwrap();
    }).await?;

    eprintln!("\nFinal response content: {}", response.content);
    println!("\n\nDone!");
    
    Ok(())
}
