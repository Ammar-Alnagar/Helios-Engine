use helios::{Agent, Config, CalculatorTool, EchoTool};
use clap::{Parser, Subcommand};
use std::io::{self, Write};

/// Helios - A powerful LLM Agent Framework
#[derive(Parser)]
#[command(name = "helios")]
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
async fn main() -> helios::Result<()> {
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
        Some(Commands::Chat { system_prompt, max_iterations }) => {
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
fn init_config(output: &str) -> helios::Result<()> {
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
    println!("2. Run: helios chat");
    println!("\nExample config structure:");
    println!("  [llm]");
    println!("  model_name = \"gpt-3.5-turbo\"");
    println!("  base_url = \"https://api.openai.com/v1\"");
    println!("  api_key = \"your-api-key-here\"");
    
    Ok(())
}

/// Send a single message and exit
async fn ask_once(config_path: &str, message: &str) -> helios::Result<()> {
    let config = load_config(config_path)?;
    
    let mut agent = Agent::builder("HeliosAgent")
        .config(config)
        .system_prompt("You are a helpful AI assistant.")
        .tool(Box::new(CalculatorTool))
        .tool(Box::new(EchoTool))
        .max_iterations(5)
        .build()?;

    let response = agent.chat(message).await?;
    println!("{}", response);
    
    Ok(())
}

/// Start an interactive chat session
async fn interactive_chat(config_path: &str, system_prompt: &str, max_iterations: usize) -> helios::Result<()> {
    println!("🚀 Helios - LLM Agent Framework");
    println!("================================\n");

    let config = load_config(config_path)?;

    // Create an agent with tools
    let mut agent = Agent::builder("HeliosAgent")
        .config(config)
        .system_prompt(system_prompt)
        .tool(Box::new(CalculatorTool))
        .tool(Box::new(EchoTool))
        .max_iterations(max_iterations)
        .build()?;

    println!("✓ Agent initialized with tools: {:?}", agent.tool_registry().list_tools());
    println!("✓ Max iterations: {}", max_iterations);
    println!("\n💬 Chat with the agent (type 'exit' to quit, 'clear' to clear history, 'help' for commands):\n");

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
                agent.clear_history();
                println!("✓ Chat history cleared\n");
                continue;
            }
            "help" => {
                print_help();
                continue;
            }
            "tools" => {
                println!("Available tools: {:?}\n", agent.tool_registry().list_tools());
                continue;
            }
            _ => {}
        }

        // Send message to agent
        match agent.chat(input).await {
            Ok(response) => {
                println!("\n🤖: {}\n", response);
            }
            Err(e) => {
                eprintln!("❌ Error: {}\n", e);
            }
        }
    }

    Ok(())
}

/// Load configuration from file
fn load_config(config_path: &str) -> helios::Result<Config> {
    match Config::from_file(config_path) {
        Ok(cfg) => {
            println!("✓ Loaded configuration from {}\n", config_path);
            
            // Check if API key is set
            if cfg.llm.api_key == "your-api-key-here" {
                eprintln!("⚠ Warning: API key not configured!");
                eprintln!("Please edit {} and set your API key.\n", config_path);
                return Err(helios::HeliosError::ConfigError(
                    "API key not configured".to_string()
                ));
            }
            
            Ok(cfg)
        }
        Err(_) => {
            eprintln!("❌ Configuration file '{}' not found!", config_path);
            eprintln!("\nTo create a new config file, run:");
            eprintln!("  helios init");
            eprintln!("\nOr specify a different config file:");
            eprintln!("  helios --config /path/to/config.toml chat\n");
            Err(helios::HeliosError::ConfigError(
                format!("Configuration file '{}' not found", config_path)
            ))
        }
    }
}

/// Print help for interactive commands
fn print_help() {
    println!("\n📖 Interactive Commands:");
    println!("  exit, quit  - Exit the chat session");
    println!("  clear       - Clear conversation history");
    println!("  tools       - List available tools");
    println!("  help        - Show this help message");
    println!();
}
