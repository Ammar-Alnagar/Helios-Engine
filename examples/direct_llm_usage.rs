/// Example: Using Helios as a crate for direct LLM calls
/// 
/// This example demonstrates how to use Helios as a library to make
/// direct calls to LLM models without using the Agent abstraction.

use helios::{LLMClient, ChatMessage, ChatSession};
use helios::config::LLMConfig;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> helios::Result<()> {
    println!("🚀 Helios Direct LLM Usage Examples\n");

    // Example 1: Simple single call
    println!("📝 Example 1: Simple Single Call");
    println!("{}", "=".repeat(50));
    simple_call().await?;
    println!();

    // Example 2: Conversation with context
    println!("💬 Example 2: Conversation with Context");
    println!("{}", "=".repeat(50));
    conversation_with_context().await?;
    println!();

    // Example 3: Different providers
    println!("🌐 Example 3: Using Different Providers");
    println!("{}", "=".repeat(50));
    different_providers_info();
    println!();

    // Example 4: Interactive chat (optional - comment out if not needed)
    println!("🎮 Example 4: Interactive Chat");
    println!("{}", "=".repeat(50));
    println!("Would you like to start an interactive chat? (y/n)");
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    
    if choice.trim().to_lowercase() == "y" {
        interactive_chat().await?;
    } else {
        println!("Skipping interactive chat.\n");
    }

    println!("✅ All examples completed!");
    Ok(())
}

/// Example 1: Simple single call to the LLM
async fn simple_call() -> helios::Result<()> {
    // Create configuration
    let llm_config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY")
            .unwrap_or_else(|_| "your-api-key-here".to_string()),
        temperature: 0.7,
        max_tokens: 2048,
    };

    // Create client
    let client = LLMClient::new(llm_config);

    // Prepare messages
    let messages = vec![
        ChatMessage::system("You are a helpful assistant that gives concise answers."),
        ChatMessage::user("What is the capital of France? Answer in one sentence."),
    ];

    // Make the call
    println!("Sending request...");
    match client.chat(messages, None).await {
        Ok(response) => {
            println!("✓ Response: {}", response.content);
        }
        Err(e) => {
            println!("✗ Error: {}", e);
            println!("  (Make sure to set OPENAI_API_KEY environment variable)");
        }
    }

    Ok(())
}

/// Example 2: Multi-turn conversation with context
async fn conversation_with_context() -> helios::Result<()> {
    let llm_config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY")
            .unwrap_or_else(|_| "your-api-key-here".to_string()),
        temperature: 0.7,
        max_tokens: 2048,
    };

    let client = LLMClient::new(llm_config);

    // Use ChatSession to manage conversation
    let mut session = ChatSession::new()
        .with_system_prompt("You are a helpful math tutor. Give brief, clear explanations.");

    // First turn
    println!("Turn 1:");
    session.add_user_message("What is 15 * 23?");
    print!("  User: What is 15 * 23?\n  ");
    
    match client.chat(session.get_messages(), None).await {
        Ok(response) => {
            session.add_assistant_message(&response.content);
            println!("Assistant: {}", response.content);
        }
        Err(e) => {
            println!("Error: {}", e);
            return Ok(());
        }
    }

    // Second turn (with context from first turn)
    println!("\nTurn 2:");
    session.add_user_message("Now divide that by 5.");
    print!("  User: Now divide that by 5.\n  ");
    
    match client.chat(session.get_messages(), None).await {
        Ok(response) => {
            session.add_assistant_message(&response.content);
            println!("Assistant: {}", response.content);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    println!("\n💡 Notice how the assistant remembered the result from the first calculation!");

    Ok(())
}

/// Example 3: Information about using different providers
fn different_providers_info() {
    println!("You can use Helios with various LLM providers:\n");

    println!("🔵 OpenAI:");
    println!("   LLMConfig {{");
    println!("       model_name: \"gpt-4\".to_string(),");
    println!("       base_url: \"https://api.openai.com/v1\".to_string(),");
    println!("       api_key: env::var(\"OPENAI_API_KEY\").unwrap(),");
    println!("       temperature: 0.7,");
    println!("       max_tokens: 2048,");
    println!("   }}\n");

    println!("🟢 Local LM Studio:");
    println!("   LLMConfig {{");
    println!("       model_name: \"local-model\".to_string(),");
    println!("       base_url: \"http://localhost:1234/v1\".to_string(),");
    println!("       api_key: \"not-needed\".to_string(),");
    println!("       temperature: 0.7,");
    println!("       max_tokens: 2048,");
    println!("   }}\n");

    println!("🦙 Ollama:");
    println!("   LLMConfig {{");
    println!("       model_name: \"llama2\".to_string(),");
    println!("       base_url: \"http://localhost:11434/v1\".to_string(),");
    println!("       api_key: \"not-needed\".to_string(),");
    println!("       temperature: 0.7,");
    println!("       max_tokens: 2048,");
    println!("   }}\n");

    println!("🔷 Azure OpenAI:");
    println!("   LLMConfig {{");
    println!("       model_name: \"gpt-35-turbo\".to_string(),");
    println!("       base_url: \"https://your-resource.openai.azure.com/...\".to_string(),");
    println!("       api_key: env::var(\"AZURE_OPENAI_KEY\").unwrap(),");
    println!("       temperature: 0.7,");
    println!("       max_tokens: 2048,");
    println!("   }}\n");
}

/// Example 4: Interactive chat session
async fn interactive_chat() -> helios::Result<()> {
    let llm_config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY")
            .unwrap_or_else(|_| "your-api-key-here".to_string()),
        temperature: 0.7,
        max_tokens: 2048,
    };

    let client = LLMClient::new(llm_config);
    let mut session = ChatSession::new()
        .with_system_prompt("You are a friendly and helpful AI assistant.");

    println!("Chat started! Type 'exit' or 'quit' to end the conversation.\n");

    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" {
            println!("\n👋 Goodbye!");
            break;
        }

        // Special commands
        if input == "clear" {
            session.clear();
            println!("🧹 Conversation cleared!\n");
            continue;
        }

        if input == "history" {
            println!("\n📜 Conversation history:");
            for (i, msg) in session.messages.iter().enumerate() {
                println!("  {}. {:?}: {}", i + 1, msg.role, msg.content);
            }
            println!();
            continue;
        }

        session.add_user_message(input);

        print!("Assistant: ");
        io::stdout().flush()?;

        match client.chat(session.get_messages(), None).await {
            Ok(response) => {
                session.add_assistant_message(&response.content);
                println!("{}\n", response.content);
            }
            Err(e) => {
                println!("\n❌ Error: {}", e);
                println!("   (Make sure OPENAI_API_KEY is set correctly)\n");
                // Remove the last user message since it failed
                session.messages.pop();
            }
        }
    }

    Ok(())
}
