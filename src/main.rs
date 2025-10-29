#[allow(unused_imports, unused_variables)]
use clap::{Parser, Subcommand};
use helios_engine::{Agent, CalculatorTool, ChatMessage, Config, EchoTool, LLMClient, LLMProvider};
use std::io::{self, Write};

/// Helper to track and display thinking tags
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

    fn process_chunk(&mut self, chunk: &str) -> Option<String> {
        let mut output = String::new();
        let mut chars = chunk.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '<' {
                // Check if this is start of a thinking tag
                let remaining: String = chars.clone().collect();
                if remaining.starts_with("thinking>") {
                    self.in_thinking = true;
                    self.thinking_buffer.clear();
                    output.push_str("\n💭 [Thinking");
                    // Skip "thinking>"
                    for _ in 0..9 {
                        chars.next();
                    }
                    continue;
                } else if remaining.starts_with("/thinking>") {
                    self.in_thinking = false;
                    output.push_str("]\n");
                    // Skip "/thinking>"
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

        if !output.is_empty() {
            Some(output)
        } else {
            None
        }
    }
}

/// Helios Engine - A powerful LLM Agent Framework
#[derive(Parser)]
#[command(name = "helios-engine")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start an interactive chat session (default)
    Chat {
        /// System prompt for the agent
        #[arg(short, long)]
        system_prompt: Option<String>,

        /// Maximum iterations for tool calls
        #[arg(short, long, default_value = "5")]
        max_iterations: usize,
    },

    /// Initialize a new config file
    Init {
        /// Path where to create the config file
        #[arg(short, long, default_value = "config.toml")]
        output: String,
    },

    /// Send a single message and exit
    Ask {
        /// The message to send
        message: String,
    },
}

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    match &cli.command {
        Some(Commands::Init { output }) => {
            init_config(output)?;
        }
        Some(Commands::Ask { message }) => {
            ask_once(&cli.config, message).await?;
        }
        Some(Commands::Chat {
            system_prompt,
            max_iterations,
        }) => {
            let sys_prompt = system_prompt.as_ref().map(|s| s.as_str()).unwrap_or(
                "You are a helpful AI assistant with access to various tools. Use them when needed to help the user."
            );
            interactive_chat(&cli.config, sys_prompt, *max_iterations).await?;
        }
        None => {
            // Default to chat command
            let sys_prompt = "You are a helpful AI assistant with access to various tools. Use them when needed to help the user.";
            interactive_chat(&cli.config, sys_prompt, 5).await?;
        }
    }

    Ok(())
}

/// Initialize a new configuration file
fn init_config(output: &str) -> helios_engine::Result<()> {
    if std::path::Path::new(output).exists() {
        println!("⚠ Configuration file '{}' already exists!", output);
        print!("Overwrite? (y/N): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let default_config = Config::new_default();
    default_config.save(output)?;

    println!("✓ Created configuration file: {}", output);
    println!("\nNext steps:");
    println!("1. Edit {} and add your API key", output);
    println!("2. Run: helios-engine chat");
    println!("\nExample config structure:");
    println!("  [llm]");
    println!("  model_name = \"gpt-3.5-turbo\"");
    println!("  base_url = \"https://api.openai.com/v1\"");
    println!("  api_key = \"your-api-key-here\"");

    Ok(())
}

/// Send a single message and exit
async fn ask_once(config_path: &str, message: &str) -> helios_engine::Result<()> {
    let config = load_config(config_path)?;

    // Use streaming for direct LLM call
    let client = LLMClient::new(config.llm);
    let messages = vec![
        ChatMessage::system("You are a helpful AI assistant."),
        ChatMessage::user(message),
    ];

    let mut tracker = ThinkingTracker::new();

    print!("🤖: ");
    io::stdout().flush().unwrap();

    let response = client
        .chat_stream(messages, None, |chunk| {
            if let Some(output) = tracker.process_chunk(chunk) {
                print!("{}", output);
                io::stdout().flush().unwrap();
            }
        })
        .await?;

    println!("\n");

    Ok(())
}

/// Start an interactive chat session
async fn interactive_chat(
    config_path: &str,
    system_prompt: &str,
    _max_iterations: usize,
) -> helios_engine::Result<()> {
    println!("🚀 Helios Engine - LLM Agent Framework");
    println!("========================================\n");

    let config = load_config(config_path)?;

    // Create LLM client for streaming
    let client = LLMClient::new(config.llm);
    let mut session = helios_engine::ChatSession::new().with_system_prompt(system_prompt);

    println!("✓ Streaming mode enabled");
    println!("✓ Thinking tags will be shown when available");
    println!("\n💬 Chat with the AI (type 'exit' to quit, 'clear' to clear history, 'help' for commands):\n");

    // Interactive chat loop
    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Handle commands
        match input.to_lowercase().as_str() {
            "exit" | "quit" => {
                println!("\n👋 Goodbye!");
                break;
            }
            "clear" => {
                session.clear();
                println!("✓ Chat history cleared\n");
                continue;
            }
            "help" => {
                print_help();
                continue;
            }
            "history" => {
                println!("\n📜 Conversation history:");
                for (i, msg) in session.messages.iter().enumerate() {
                    println!("  {}. {:?}: {}", i + 1, msg.role, msg.content);
                }
                println!();
                continue;
            }
            _ => {}
        }

        // Add user message to session
        session.add_user_message(input);

        // Stream response
        let mut tracker = ThinkingTracker::new();
        print!("\n🤖: ");
        io::stdout().flush()?;

        match client
            .chat_stream(session.get_messages(), None, |chunk| {
                if let Some(output) = tracker.process_chunk(chunk) {
                    print!("{}", output);
                    io::stdout().flush().unwrap();
                }
            })
            .await
        {
            Ok(response) => {
                session.add_assistant_message(&response.content);
                println!("\n");
            }
            Err(e) => {
                eprintln!("\n❌ Error: {}\n", e);
                // Remove the last user message since it failed
                session.messages.pop();
            }
        }
    }

    Ok(())
}

/// Load configuration from file
fn load_config(config_path: &str) -> helios_engine::Result<Config> {
    match Config::from_file(config_path) {
        Ok(cfg) => {
            println!("✓ Loaded configuration from {}\n", config_path);

            // Check if API key is set
            if cfg.llm.api_key == "your-api-key-here" {
                eprintln!("⚠ Warning: API key not configured!");
                eprintln!("Please edit {} and set your API key.\n", config_path);
                return Err(helios_engine::HeliosError::ConfigError(
                    "API key not configured".to_string(),
                ));
            }

            Ok(cfg)
        }
        Err(_) => {
            eprintln!("❌ Configuration file '{}' not found!", config_path);
            eprintln!("\nTo create a new config file, run:");
            eprintln!("  helios-engine init");
            eprintln!("\nOr specify a different config file:");
            eprintln!("  helios-engine --config /path/to/config.toml chat\n");
            Err(helios_engine::HeliosError::ConfigError(format!(
                "Configuration file '{}' not found",
                config_path
            )))
        }
    }
}

/// Print help for interactive commands
fn print_help() {
    println!("\n📖 Interactive Commands:");
    println!("  exit, quit  - Exit the chat session");
    println!("  clear       - Clear conversation history");
    println!("  history     - Show conversation history");
    println!("  help        - Show this help message");
    println!("\n💡 Features:");
    println!("  • Streaming responses for real-time output");
    println!("  • Thinking tags displayed when model uses them");
    println!("  • Full conversation context maintained");
    println!();
}
