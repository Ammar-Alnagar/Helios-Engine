/// Example: Using streaming responses with Helios Engine
/// 
/// This example demonstrates how to use the streaming API to get
/// real-time responses from the LLM, including detection of thinking tags.

use helios_engine::{LLMClient, ChatMessage, ChatSession};
use helios_engine::config::LLMConfig;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Helios Engine - Streaming Example");
    println!("=====================================\n");

    // Setup LLM configuration
    let llm_config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY")
            .unwrap_or_else(|_| "your-api-key-here".to_string()),
        temperature: 0.7,
        max_tokens: 2048,
    };

    let client = LLMClient::new(llm_config);

    println!("Example 1: Simple Streaming Response");
    println!("======================================\n");
    
    let messages = vec![
        ChatMessage::system("You are a helpful assistant."),
        ChatMessage::user("Write a short poem about coding."),
    ];

    print!("Assistant: ");
    io::stdout().flush()?;

    let response = client.chat_stream(messages, None, |chunk| {
        print!("{}", chunk);
        io::stdout().flush().unwrap();
    }).await?;

    println!("\n\n");

    println!("Example 2: Interactive Streaming Chat");
    println!("======================================\n");

    let mut session = ChatSession::new()
        .with_system_prompt("You are a helpful coding assistant.");

    let questions = vec![
        "What is Rust?",
        "What are its main benefits?",
        "Show me a simple example.",
    ];

    for question in questions {
        println!("User: {}", question);
        session.add_user_message(question);

        print!("Assistant: ");
        io::stdout().flush()?;

        let response = client.chat_stream(session.get_messages(), None, |chunk| {
            print!("{}", chunk);
            io::stdout().flush().unwrap();
        }).await?;

        session.add_assistant_message(&response.content);
        println!("\n");
    }

    println!("\nExample 3: Streaming with Thinking Tags");
    println!("=========================================\n");
    println!("When using models that support thinking tags (like o1),");
    println!("you can detect and display them during streaming.\n");

    struct ThinkingTracker {
        in_thinking: bool,
        thinking_buffer: String,
    }

    impl ThinkingTracker {
        fn new() -> Self {
            Self {
                in_thinking: false,
                thinking_buffer: String::new(),
            }
        }

        fn process_chunk(&mut self, chunk: &str) -> String {
            let mut output = String::new();
            let mut chars = chunk.chars().peekable();

            while let Some(c) = chars.next() {
                if c == '<' {
                    let remaining: String = chars.clone().collect();
                    if remaining.starts_with("thinking>") {
                        self.in_thinking = true;
                        self.thinking_buffer.clear();
                        output.push_str("\n💭 [Thinking");
                        for _ in 0..9 {
                            chars.next();
                        }
                        continue;
                    } else if remaining.starts_with("/thinking>") {
                        self.in_thinking = false;
                        output.push_str("]\n");
                        for _ in 0..10 {
                            chars.next();
                        }
                        continue;
                    }
                }

                if self.in_thinking {
                    self.thinking_buffer.push(c);
                    if self.thinking_buffer.len() % 3 == 0 {
                        output.push('.');
                    }
                } else {
                    output.push(c);
                }
            }

            output
        }
    }

    let messages = vec![
        ChatMessage::user("Solve this problem: What is 15 * 234 + 89?"),
    ];

    let mut tracker = ThinkingTracker::new();
    print!("Assistant: ");
    io::stdout().flush()?;

    let _response = client.chat_stream(messages, None, |chunk| {
        let output = tracker.process_chunk(chunk);
        print!("{}", output);
        io::stdout().flush().unwrap();
    }).await?;

    println!("\n\n✅ Streaming examples completed!");
    println!("\nKey benefits of streaming:");
    println!("  • Real-time response display");
    println!("  • Better user experience for long responses");
    println!("  • Ability to show thinking/reasoning process");
    println!("  • Early cancellation possible (future feature)");

    Ok(())
}
