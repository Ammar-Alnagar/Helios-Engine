//! # Example: Comparing Agents With and Without ReAct
//!
//! This example demonstrates the difference between standard agents and ReAct agents
//! by running the same queries through both and comparing their approaches.

use helios_engine::{Agent, CalculatorTool, Config, EchoTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🔬 Helios Engine - ReAct Comparison Demo");
    println!("=========================================\n");

    let config1 = Config::from_file("config.toml")?;
    let config2 = Config::from_file("config.toml")?;

    // Create two identical agents, one with ReAct and one without
    let mut standard_agent = Agent::builder("StandardAgent")
        .config(config1)
        .system_prompt("You are a helpful assistant with access to tools.")
        .tools(vec![Box::new(CalculatorTool), Box::new(EchoTool)])
        .build()
        .await?;

    let mut react_agent = Agent::builder("ReActAgent")
        .config(config2)
        .system_prompt("You are a helpful assistant with access to tools.")
        .tools(vec![Box::new(CalculatorTool), Box::new(EchoTool)])
        .react() // The only difference!
        .build()
        .await?;

    println!(
        "Tools available: {:?}\n",
        standard_agent.tool_registry().list_tools()
    );

    // Test Case 1: Simple calculation
    println!("═══════════════════════════════════════════════════════════");
    println!("Test Case 1: Simple Mathematical Calculation");
    println!("═══════════════════════════════════════════════════════════\n");

    let query1 = "What is 25 * 8?";
    println!("Query: {}\n", query1);

    println!("--- STANDARD AGENT ---");
    let response1 = standard_agent.chat(query1).await?;
    println!("Response: {}\n", response1);

    println!("--- REACT AGENT ---");
    let response2 = react_agent.chat(query1).await?;
    println!("Response: {}\n", response2);

    // Test Case 2: Multi-step problem
    println!("═══════════════════════════════════════════════════════════");
    println!("Test Case 2: Multi-Step Problem");
    println!("═══════════════════════════════════════════════════════════\n");

    let query2 = "Calculate (15 + 25) * 3, then echo the result";
    println!("Query: {}\n", query2);

    println!("--- STANDARD AGENT ---");
    let response3 = standard_agent.chat(query2).await?;
    println!("Response: {}\n", response3);

    println!("--- REACT AGENT ---");
    let response4 = react_agent.chat(query2).await?;
    println!("Response: {}\n", response4);

    // Test Case 3: Complex multi-tool task
    println!("═══════════════════════════════════════════════════════════");
    println!("Test Case 3: Complex Multi-Tool Task");
    println!("═══════════════════════════════════════════════════════════\n");

    let query3 =
        "First calculate 100 / 4, then multiply that by 3, and finally echo the final answer";
    println!("Query: {}\n", query3);

    println!("--- STANDARD AGENT ---");
    let response5 = standard_agent.chat(query3).await?;
    println!("Response: {}\n", response5);

    println!("--- REACT AGENT ---");
    let response6 = react_agent.chat(query3).await?;
    println!("Response: {}\n", response6);

    // Summary
    println!("═══════════════════════════════════════════════════════════");
    println!("📊 Comparison Summary");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("STANDARD AGENT:");
    println!("  ✓ Faster execution (no reasoning overhead)");
    println!("  ✓ Direct tool usage");
    println!("  ✗ No visible thought process");
    println!("  ✗ May miss planning opportunities\n");

    println!("REACT AGENT:");
    println!("  ✓ Shows reasoning process (💭 ReAct Reasoning)");
    println!("  ✓ Systematic approach to problems");
    println!("  ✓ Better for complex tasks");
    println!("  ✗ Slightly slower (extra LLM call)\n");

    println!("WHEN TO USE:");
    println!("  → Standard: Simple, direct tasks where speed matters");
    println!("  → ReAct: Complex tasks, debugging, when you want transparency\n");

    Ok(())
}
