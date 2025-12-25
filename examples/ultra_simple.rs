//! # Ultra Simple Example
//!
//! This example shows the absolute EASIEST way to use Helios Engine.
//! Just a few lines to create an agent and chat!

use helios_engine::{Agent, CalculatorTool, Config};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Ultra Simple Helios Example\n");

    // ========== SIMPLEST AGENT CREATION ==========
    println!("1️⃣  Creating an agent - shortest possible syntax:\n");

    // One-liner: Create agent with auto config
    let mut agent = Agent::builder("Helper")
        .auto_config()
        .prompt("You are helpful and concise.")
        .build()
        .await?;

    println!("✓ Agent created!\n");

    // ========== SIMPLEST CHAT ==========
    println!("2️⃣  Asking questions - simplest possible:\n");

    // Use .ask() instead of .chat() for more natural syntax
    let answer = agent.ask("What is 2+2?").await?;
    println!("Q: What is 2+2?\nA: {}\n", answer);

    // ========== SIMPLEST CONFIG ==========
    println!("3️⃣  Creating config with shortest syntax:\n");

    // Ultra-short config creation
    let config = Config::builder()
        .m("gpt-4") // .m() is shorthand for .model()
        .key("your-api-key") // .key() is shorthand for .api_key()
        .temp(0.8) // .temp() is shorthand for .temperature()
        .tokens(1024) // .tokens() is shorthand for .max_tokens()
        .build();

    println!("✓ Config created with ultra-short syntax!\n");

    // ========== SIMPLEST AGENT WITH TOOLS ==========
    println!("4️⃣  Agent with tools - simplest way:\n");

    let mut calc_agent = Agent::builder("Calculator")
        .auto_config()
        .prompt("You are a math expert.")
        .with_tool(Box::new(CalculatorTool)) // Add single tool
        .build()
        .await?;

    let result = calc_agent.ask("Calculate 15 * 7 + 5").await?;
    println!("Q: Calculate 15 * 7 + 5\nA: {}\n", result);

    // ========== SIMPLEST QUICK AGENT ==========
    println!("5️⃣  Quick agent - one method call:\n");

    // Agent::quick() creates agent in ONE LINE with auto config!
    let mut quick_agent = Agent::quick("QuickBot").await?;
    let quick_answer = quick_agent.ask("Say hello!").await?;
    println!("Response: {}\n", quick_answer);

    // ========== SIMPLEST CHAT MESSAGES ==========
    println!("6️⃣  Creating messages - super short syntax:\n");

    use helios_engine::ChatMessage;

    // Short aliases for message creation
    let sys_msg = ChatMessage::sys("You are helpful"); // .sys() not .system()
    let user_msg = ChatMessage::msg("Hello there"); // .msg() not .user()
    let reply_msg = ChatMessage::reply("Hi! How can I help?"); // .reply() not .assistant()

    println!("✓ Messages created with ultra-short syntax!\n");

    // ========== SHORTEST AUTOFOREST ==========
    println!("7️⃣  AutoForest - simplest multi-agent orchestration:\n");

    use helios_engine::AutoForest;

    let mut forest = AutoForest::new(Config::builder().m("gpt-4").build())
        .with_tools(vec![Box::new(CalculatorTool)])
        .build()
        .await?;

    // Use .run() for shortest syntax
    let forest_result = forest.run("Analyze these numbers: 10, 20, 30, 40").await?;
    println!("Forest Result:\n{}\n", forest_result);

    // ========== COMPARISON TABLE ==========
    println!("📊 Syntax Comparison - Short vs Long:\n");
    println!("┌─────────────────────┬──────────────────────┬─────────────────┐");
    println!("│ Operation           │ Short Syntax         │ Long Syntax      │");
    println!("├─────────────────────┼──────────────────────┼─────────────────┤");
    println!("│ Create Agent        │ Agent::quick()       │ Agent::builder()│");
    println!("│ Ask Question        │ .ask()               │ .chat()          │");
    println!("│ System Prompt       │ .prompt()            │ .system_prompt() │");
    println!("│ Config Model        │ .m()                 │ .model()         │");
    println!("│ Config Key          │ .key()               │ .api_key()       │");
    println!("│ Config Temp         │ .temp()              │ .temperature()   │");
    println!("│ Config Tokens       │ .tokens()            │ .max_tokens()    │");
    println!("│ System Message      │ ChatMessage::sys()   │ ChatMessage::system()");
    println!("│ User Message        │ ChatMessage::msg()   │ ChatMessage::user()");
    println!("│ Assistant Message   │ ChatMessage::reply() │ ChatMessage::assistant()");
    println!("│ AutoForest Execute  │ .run()               │ .execute_task()  │");
    println!("└─────────────────────┴──────────────────────┴─────────────────┘\n");

    println!("✅ All examples completed!");
    println!("💡 Tip: Mix and match short and long syntax based on your preference!");

    Ok(())
}
