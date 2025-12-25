//! # Example: Using ReAct for Debugging
//!
//! This example shows how ReAct mode helps with debugging and understanding
//! how agents approach problems.

use helios_engine::{Agent, CalculatorTool, Config, FileReadTool, JsonParserTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🔍 Helios Engine - ReAct for Debugging");
    println!("=======================================\n");

    let config = Config::from_file("config.toml")?;

    // Create a ReAct agent with verbose reasoning
    let debug_prompt = r#"Debug this task step by step. For each step, explain:

1. CURRENT STATE: What information do I have?
2. NEXT ACTION: What should I do next?
3. REASONING: Why is this the right action?
4. EXPECTED RESULT: What should happen?
5. VALIDATION: How will I know if it worked?

Be extremely detailed in your thinking."#;

    let mut debug_agent = Agent::builder("DebugAgent")
        .config(config)
        .system_prompt("You are a debugging assistant who explains every decision.")
        .tools(vec![
            Box::new(CalculatorTool),
            Box::new(JsonParserTool),
            Box::new(FileReadTool),
        ])
        .react_with_prompt(debug_prompt)
        .max_iterations(15) // Allow more iterations for complex debugging
        .build()
        .await?;

    println!(
        "Available tools: {:?}\n",
        debug_agent.tool_registry().list_tools()
    );

    // Scenario 1: Tracing calculation steps
    println!("═══════════════════════════════════════════════════════════");
    println!("Scenario 1: Trace Complex Calculation");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("Problem: Calculate the compound interest formula result");
    println!("Formula: A = P(1 + r)^n where P=1000, r=0.05, n=3\n");

    let response = debug_agent
        .chat("Calculate compound interest: Principal=1000, rate=0.05, time=3 years. Use A = P * (1 + r)^n")
        .await?;
    println!("\nAgent: {}\n", response);

    // Scenario 2: Understanding tool selection
    println!("═══════════════════════════════════════════════════════════");
    println!("Scenario 2: Tool Selection Reasoning");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("Task: Parse JSON and extract a value, then perform calculation\n");

    let response = debug_agent
        .chat(r#"Parse this JSON: {"price": 25.50, "quantity": 4} and calculate the total cost"#)
        .await?;
    println!("\nAgent: {}\n", response);

    // Scenario 3: Error recovery reasoning
    println!("═══════════════════════════════════════════════════════════");
    println!("Scenario 3: Multi-Step Problem Solving");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("Task: Calculate average of a series of operations\n");

    let response = debug_agent
        .chat("Calculate: (10 * 5) + (20 * 3) + (15 * 2), then divide by 3 to get the average")
        .await?;
    println!("\nAgent: {}\n", response);

    // Explanation
    println!("═══════════════════════════════════════════════════════════");
    println!("💡 Debugging Benefits");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("ReAct mode helps you:");
    println!("  1. 🔍 See exactly what the agent is thinking");
    println!("  2. 🎯 Understand why it chose specific tools");
    println!("  3. 📋 Follow the step-by-step execution plan");
    println!("  4. 🐛 Identify where reasoning might go wrong");
    println!("  5. 🔧 Optimize prompts based on visible thinking\n");

    println!("Tips for debugging with ReAct:");
    println!("  • Use detailed custom prompts for more verbose reasoning");
    println!("  • Increase max_iterations for complex tasks");
    println!("  • Watch the '💭 ReAct Reasoning' output carefully");
    println!("  • Compare reasoning across different queries");
    println!("  • Adjust system prompts based on reasoning patterns\n");

    Ok(())
}
