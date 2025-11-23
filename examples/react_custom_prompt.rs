//! # Example: ReAct with Custom Reasoning Prompt
//!
//! This example demonstrates how to use ReAct mode with custom reasoning prompts
//! tailored to specific domains or tasks.

use helios_engine::{Agent, CalculatorTool, Config, FileReadTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🧠 Helios Engine - ReAct with Custom Prompts");
    println!("=============================================\n");

    let config = Config::from_file("config.toml")?;

    // Example 1: Math-focused reasoning prompt
    println!("═══════════════════════════════════════════════════════════");
    println!("Example 1: Mathematical Problem Solver");
    println!("═══════════════════════════════════════════════════════════\n");

    let math_prompt = r#"As a mathematical problem solver, analyze this systematically:

1. IDENTIFY: What mathematical operations are required?
2. DECOMPOSE: Break complex calculations into simple steps
3. ORDER: Determine the correct order of operations (PEMDAS/BODMAS)
4. PLAN: List which calculator functions to use and in what sequence
5. VERIFY: Consider how to check the answer

Provide your mathematical reasoning clearly."#;

    let mut math_agent = Agent::builder("MathExpert")
        .config(config.clone())
        .system_prompt("You are a mathematical expert who thinks carefully about calculations.")
        .tool(Box::new(CalculatorTool))
        .react_with_prompt(math_prompt)
        .build()
        .await?;

    println!("User: Calculate ((15 * 8) + (20 * 3)) / 2\n");
    let response = math_agent
        .chat("Calculate ((15 * 8) + (20 * 3)) / 2")
        .await?;
    println!("\nAgent: {}\n", response);

    // Example 2: Data analysis reasoning prompt
    println!("═══════════════════════════════════════════════════════════");
    println!("Example 2: Data Analysis Agent");
    println!("═══════════════════════════════════════════════════════════\n");

    let data_prompt = r#"As a data analyst, approach this task methodically:

1. UNDERSTAND: What data or files are we working with?
2. EXTRACT: What information needs to be retrieved?
3. PROCESS: What transformations or calculations are needed?
4. TOOLS: Which tools should I use and in what order?
5. OUTPUT: What format should the final answer take?

Think through the data pipeline step by step."#;

    let mut data_agent = Agent::builder("DataAnalyst")
        .config(config.clone())
        .system_prompt("You are a data analyst who carefully plans data processing tasks.")
        .tools(vec![Box::new(FileReadTool), Box::new(CalculatorTool)])
        .react_with_prompt(data_prompt)
        .build()
        .await?;

    println!("User: If I have numbers 10, 20, 30, 40, 50, what's their average?\n");
    let response = data_agent
        .chat("If I have numbers 10, 20, 30, 40, 50, what's their average?")
        .await?;
    println!("\nAgent: {}\n", response);

    // Example 3: Task planning reasoning prompt
    println!("═══════════════════════════════════════════════════════════");
    println!("Example 3: Task Planning Agent");
    println!("═══════════════════════════════════════════════════════════\n");

    let planning_prompt = r#"As a task planning expert, organize this systematically:

1. GOAL: What is the end objective?
2. PREREQUISITES: What information do I already have?
3. DEPENDENCIES: What needs to happen before what?
4. RESOURCES: What tools are available to me?
5. STEPS: Create a numbered action plan
6. CONTINGENCY: What could go wrong?

Plan the execution strategy carefully."#;

    let mut planning_agent = Agent::builder("TaskPlanner")
        .config(config.clone())
        .system_prompt("You are a strategic planner who breaks down complex tasks.")
        .tool(Box::new(CalculatorTool))
        .react_with_prompt(planning_prompt)
        .build()
        .await?;

    println!("User: I need to calculate the total cost: 5 items at $12.50 each, plus 8% tax\n");
    let response = planning_agent
        .chat("I need to calculate the total cost: 5 items at $12.50 each, plus 8% tax")
        .await?;
    println!("\nAgent: {}\n", response);

    // Example 4: Scientific reasoning prompt
    println!("═══════════════════════════════════════════════════════════");
    println!("Example 4: Scientific Reasoning Agent");
    println!("═══════════════════════════════════════════════════════════\n");

    let scientific_prompt = r#"Apply the scientific method to this problem:

1. OBSERVATION: What is being asked?
2. HYPOTHESIS: What approach should work?
3. VARIABLES: What factors are involved?
4. METHOD: What tools and operations are needed?
5. PREDICTION: What result do we expect?
6. VERIFICATION: How can we validate the answer?

Use rigorous scientific thinking."#;

    let mut science_agent = Agent::builder("Scientist")
        .config(config)
        .system_prompt("You are a scientist who applies rigorous methodology.")
        .tool(Box::new(CalculatorTool))
        .react_with_prompt(scientific_prompt)
        .build()
        .await?;

    println!("User: If velocity = 30 m/s and time = 4 seconds, what's the distance?\n");
    let response = science_agent
        .chat("If velocity = 30 m/s and time = 4 seconds, what's the distance?")
        .await?;
    println!("\nAgent: {}\n", response);

    println!("═══════════════════════════════════════════════════════════");
    println!("✅ Custom Prompt Demo Complete!");
    println!("═══════════════════════════════════════════════════════════");
    println!("\nKey Takeaways:");
    println!("  • Custom prompts tailor reasoning to specific domains");
    println!("  • Different prompts optimize for different task types");
    println!("  • Use .react_with_prompt() for domain-specific reasoning");
    println!("  • Each agent can have its own reasoning style\n");

    Ok(())
}
