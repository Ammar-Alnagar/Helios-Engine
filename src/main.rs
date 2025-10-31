//! # Helios Engine CLI
//!
//! This is the command-line interface for the Helios Engine.
//! It provides commands for interactive chat, asking single questions,
//! and initializing the configuration.

#![allow(unused_imports)]
#![allow(unused_variables)]

use clap::{Parser, Subcommand};
use helios_engine::{ChatMessage, Config, LLMClient};
use std::io::{self, Write};

/// A helper struct to track and display thinking tags in streamed responses.
struct ThinkingTracker {
    in_thinking: bool,
    thinking_buffer: String,
}

impl ThinkingTracker {
    /// Creates a new `ThinkingTracker`.
    fn new() -> Self {
        Self {
            in_thinking: false,
            thinking_buffer: String::new(),
        }
    }

    /// Processes a chunk of a streamed response and returns the processed output.
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
                } else if remaining.starts_with("think>") {
                    self.in_thinking = true;
                    self.thinking_buffer.clear();
                    output.push_str("\n💭 [Thinking");
                    // Skip "think>"
                    for _ in 0..6 {
                        chars.next();
                    }
                    continue;
                } else if remaining.starts_with("/think>") {
                    self.in_thinking = false;
                    output.push_str("]\n");
                    // Skip "/think>"
                    for _ in 0..7 {
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

/// Processes thinking tags in the content of a non-streaming response.
#[allow(dead_code)]
fn process_thinking_tags_in_content(content: &str) -> String {
    let mut result = String::new();
    let mut in_thinking = false;
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            // Check if this is a thinking tag
            let remaining: String = chars.clone().collect();
            if remaining.starts_with("thinking>") {
                in_thinking = true;
                // Skip "thinking>"
                for _ in 0..9 {
                    chars.next();
                }
                continue;
            } else if remaining.starts_with("/thinking>") {
                in_thinking = false;
                // Skip "/thinking>"
                for _ in 0..10 {
                    chars.next();
                }
                continue;
            } else if remaining.starts_with("think>") {
                in_thinking = true;
                // Skip "think>"
                for _ in 0..6 {
                    chars.next();
                }
                continue;
            } else if remaining.starts_with("/think>") {
                in_thinking = false;
                // Skip "/think>"
                for _ in 0..7 {
                    chars.next();
                }
                continue;
            }
        }

        if !in_thinking {
            result.push(c);
        }
    }

    result
}

/// The command-line interface for the Helios Engine.
#[derive(Parser)]
#[command(name = "helios-engine")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the configuration file.
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Enable verbose logging.
    #[arg(short, long)]
    verbose: bool,

    /// The LLM mode to use.
    #[arg(long, default_value = "auto")]
    mode: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

/// The subcommands for the Helios Engine CLI.
#[derive(Subcommand)]
enum Commands {
    /// Start an interactive chat session.
    Chat {
        /// The system prompt for the agent.
        #[arg(short, long)]
        system_prompt: Option<String>,

        /// The maximum number of iterations for tool calls.
        #[arg(short, long, default_value = "5")]
        max_iterations: usize,
    },

    /// Initialize a new configuration file.
    Init {
        /// The path where to create the configuration file.
        #[arg(short, long, default_value = "config.toml")]
        output: String,
    },

    /// Send a single message and exit.
    Ask {
        /// The message to send.
        message: String,
    },
}

/// The main entry point for the Helios Engine CLI.
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
            ask_once(&cli.config, message, &cli.mode).await?;
        }
        Some(Commands::Chat {
            system_prompt,
            max_iterations,
        }) => {
            let sys_prompt = system_prompt.as_ref().map(|s| s.as_str()).unwrap_or(
                "You are a helpful AI assistant with access to various tools. Use them when needed to help the user."
            );
            interactive_chat(&cli.config, sys_prompt, *max_iterations, &cli.mode).await?;
        }
        None => {
            // Default to chat command
            let sys_prompt = "You are a helpful AI assistant with access to various tools. Use them when needed to help the user.";
            interactive_chat(&cli.config, sys_prompt, 5, &cli.mode).await?;
        }
    }

    Ok(())
}

/// Initializes a new configuration file.
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

/// Sends a single message to the LLM and exits.
async fn ask_once(config_path: &str, message: &str, mode: &str) -> helios_engine::Result<()> {
    let mut config = load_config(config_path)?;
    apply_mode_override(&mut config, mode);

    // Determine if we're using local or remote
    let is_local = config.local.is_some();

    // Use streaming for direct LLM call
    let provider_type = if is_local {
        helios_engine::llm::LLMProviderType::Local(config.local.unwrap())
    } else {
        helios_engine::llm::LLMProviderType::Remote(config.llm)
    };
    let client = LLMClient::new(provider_type).await?;
    let messages = vec![
        ChatMessage::system("You are a helpful AI assistant. Provide direct, concise answers without internal reasoning or thinking tags."),
        ChatMessage::user(message),
    ];

    let mut tracker = ThinkingTracker::new();

    print!("🤖: ");
    io::stdout().flush().unwrap();

    // Use streaming for both local and remote models
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

/// Starts an interactive chat session with the LLM.
async fn interactive_chat(
    config_path: &str,
    system_prompt: &str,
    _max_iterations: usize,
    mode: &str,
) -> helios_engine::Result<()> {
    println!("🚀 Helios Engine - LLM Agent Framework");
    println!("========================================\n");

    let mut config = load_config(config_path)?;
    apply_mode_override(&mut config, mode);

    // Create LLM client for streaming
    let provider_type = if config.local.is_some() {
        helios_engine::llm::LLMProviderType::Local(config.local.unwrap())
    } else {
        helios_engine::llm::LLMProviderType::Remote(config.llm)
    };
    let client = LLMClient::new(provider_type).await?;
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
            "summary" => {
                println!("\n📊 Session Summary:");
                println!("{}", session.get_summary());
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

/// Loads the configuration from a file.
fn load_config(config_path: &str) -> helios_engine::Result<Config> {
    match Config::from_file(config_path) {
        Ok(cfg) => {
            println!("✓ Loaded configuration from {}\n", config_path);
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

/// Applies the mode override to the configuration.
fn apply_mode_override(config: &mut Config, mode: &str) {
    match mode {
        "online" => {
            // Force online mode by removing local config
            config.local = None;
            println!("🌐 Online mode: Using remote API");

            // Check if API key is set for online mode
            if config.llm.api_key == "your-api-key-here" {
                eprintln!("⚠ Warning: API key not configured!");
                eprintln!("Please edit your config file and set your API key.\n");
                std::process::exit(1);
            }
        }
        "offline" => {
            // Force offline mode - require local config to be present
            if config.local.is_none() {
                eprintln!("❌ Offline mode requested but no [local] section found in config");
                eprintln!("💡 Add a [local] section to your config.toml for offline mode");
                std::process::exit(1);
            }
            println!("🏠 Offline mode: Using local models");
        }
        "auto" => {
            // Use existing logic (local if present, otherwise remote)
            if config.local.is_some() {
                println!("🔄 Auto mode: Using local models (configured)");
            } else {
                println!("🔄 Auto mode: Using remote API (no local config)");
                // Check if API key is set for remote mode in auto mode
                if config.llm.api_key == "your-api-key-here" {
                    eprintln!("⚠ Warning: API key not configured!");
                    eprintln!("Please edit your config file and set your API key.\n");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!(
                "❌ Invalid mode '{}'. Valid options: auto, online, offline",
                mode
            );
            std::process::exit(1);
        }
    }
}

/// Prints the help message for interactive commands.
fn print_help() {
    println!("\n📖 Interactive Commands:");
    println!("  exit, quit  - Exit the chat session");
    println!("  clear       - Clear conversation history");
    println!("  history     - Show conversation history");
    println!("  summary     - Show session summary with metadata");
    println!("  help        - Show this help message");
    println!("\n💡 Features:");
    println!("  • Streaming responses for real-time output (local & remote)");
    println!("  • Thinking tags displayed when model uses them");
    println!("  • Full conversation context maintained");
    println!("  • Session memory for tracking conversation state");
    println!();
}
