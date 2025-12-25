//! # AutoForest Demo
//!
//! This example demonstrates the AutoForest feature - automatic orchestration of agent forests.
//! AutoForest intelligently spawns specialized agents to tackle complex tasks.
//!
//! The example shows:
//! 1. Creating an AutoForest orchestrator
//! 2. Submitting a complex task
//! 3. Inspecting the generated orchestration plan
//! 4. Getting the synthesized results

use helios_engine::{AutoForest, CalculatorTool, Config};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🌲 AutoForest - Intelligent Agent Orchestration Demo\n");
    println!("====================================================\n");

    // Step 1: Load or create configuration
    println!("📋 Creating configuration...");
    let config = Config::builder().temperature(0.7).max_tokens(2048).build();
    println!("✓ Configuration ready\n");

    // Step 2: Create AutoForest with available tools
    println!("🔧 Initializing AutoForest with tools...");
    let mut auto_forest = AutoForest::new(config)
        .with_tools(vec![
            Box::new(CalculatorTool),
            // Additional tools could be added here
        ])
        .build()
        .await?;
    println!("✓ AutoForest initialized\n");

    // Step 3: Define a complex task
    let task = r#"
    I need to analyze a business problem:
    1. A company has Q3 revenue data showing mixed results across 5 product categories
    2. I need to understand which categories are underperforming
    3. Calculate the performance metrics for each category
    4. Identify trends and predict Q4 performance
    5. Recommend which categories need attention
    "#;

    println!("📝 Submitting task to AutoForest...");
    println!("Task: {}\n", task.trim());

    // Step 4: Execute the task
    println!("🚀 AutoForest is orchestrating agent deployment...\n");
    let result = auto_forest.execute_task(task).await?;

    // Step 5: Display the orchestration plan
    println!("📊 Orchestration Plan Generated:");
    println!("================================\n");

    if let Some(plan) = auto_forest.orchestration_plan() {
        println!("Task: {}", plan.task.trim());
        println!("Number of Agents Spawned: {}", plan.num_agents);
        println!("Planning Reasoning: {}\n", plan.reasoning);

        println!("Agent Configurations:");
        println!("-----------------");
        for (i, agent_config) in plan.agents.iter().enumerate() {
            println!(
                "Agent {}: {} ({})",
                i + 1,
                agent_config.name,
                agent_config.role
            );
            println!("  Prompt: {}", agent_config.system_prompt);
            if !agent_config.tool_indices.is_empty() {
                println!("  Tools: {:?}", agent_config.tool_indices);
            }
            println!();
        }

        println!("Task Breakdown:");
        println!("--------------");
        for (agent_name, subtask) in &plan.task_breakdown {
            println!("• {} → {}", agent_name, subtask);
        }
        println!();
    }

    // Step 6: Display spawned agents
    println!("🤖 Spawned Agents:");
    println!("-----------------");
    let spawned = auto_forest.spawned_agents();
    println!("Total agents created: {}\n", spawned.len());

    for (i, spawned_agent) in spawned.iter().enumerate() {
        println!("Agent {}: {}", i + 1, spawned_agent.config.name);
        println!("  Role: {}", spawned_agent.config.role);
        if let Some(result) = &spawned_agent.result {
            println!("  Result available: Yes ({} chars)", result.len());
        } else {
            println!("  Result: Pending");
        }
        println!();
    }

    // Step 7: Display the final synthesized result
    println!("📈 Final Synthesized Result:");
    println!("==========================\n");
    println!("{}\n", result);

    // Step 8: Demonstrate a second task with different complexity
    println!("\n🔄 Executing a second task with different complexity...\n");

    let simple_task = "Calculate the average of these numbers: 100, 200, 300, 400, 500. Then tell me what percentage each is of the total.";

    println!("Task: {}\n", simple_task);
    println!("🚀 Processing...\n");

    let result2 = auto_forest.execute_task(simple_task).await?;

    println!("Result:\n{}\n", result2);

    if let Some(plan) = auto_forest.orchestration_plan() {
        println!(
            "This task resulted in {} agent(s) - a simpler orchestration plan.",
            plan.num_agents
        );
        println!("Reasoning: {}\n", plan.reasoning);
    }

    println!("✅ AutoForest demo completed!");
    println!("=============================");
    println!("\n💡 Key Features Demonstrated:");
    println!("  • Automatic agent spawning based on task complexity");
    println!("  • Specialized agent prompt generation");
    println!("  • Task breakdown across agents");
    println!("  • Result aggregation and synthesis");
    println!("  • Flexible orchestration for varying task complexity");

    Ok(())
}
