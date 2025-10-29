use helios::{Agent, Config, CalculatorTool, EchoTool};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> helios::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 Helios - LLM Agent Framework");
    println!("================================\n");

    // Load or create config
    let config = match Config::from_file("config.toml") {
        Ok(cfg) => {
            println!("✓ Loaded configuration from config.toml");
            cfg
        }
        Err(_) => {
            println!("! No config.toml found, creating default configuration...");
            let default_config = Config::new_default();
            default_config.save("config.toml")?;
            println!("✓ Created config.toml - Please update with your API key");
            println!("\nExiting. Please configure your API key in config.toml and run again.\n");
            return Ok(());
        }
    };

    // Check if API key is set
    if config.llm.api_key == "your-api-key-here" {
        println!("⚠ Please set your API key in config.toml");
        println!("\nExiting. Please configure your API key in config.toml and run again.\n");
        return Ok(());
    }

    // Create an agent with tools
    let mut agent = Agent::builder("HeliosAgent")
        .config(config)
        .system_prompt("You are a helpful AI assistant with access to various tools. Use them when needed to help the user.")
        .tool(Box::new(CalculatorTool))
        .tool(Box::new(EchoTool))
        .max_iterations(5)
        .build()?;

    println!("\n✓ Agent initialized with tools: {:?}", agent.tool_registry().list_tools());
    println!("\n💬 Chat with the agent (type 'exit' to quit, 'clear' to clear history):\n");

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

        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            println!("\n👋 Goodbye!");
            break;
        }

        if input.eq_ignore_ascii_case("clear") {
            agent.clear_history();
            println!("✓ Chat history cleared\n");
            continue;
        }

        match agent.chat(input).await {
            Ok(response) => {
                println!("\nAgent: {}\n", response);
            }
            Err(e) => {
                eprintln!("Error: {}\n", e);
            }
        }
    }

    Ok(())
}
