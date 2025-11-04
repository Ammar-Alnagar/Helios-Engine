//! # Enhanced Forest of Agents with Coordinator and Shared Memory
//!
//! This example demonstrates the enhanced Forest of Agents feature with:
//! - A coordinator agent that creates detailed task plans
//! - Agents updating shared task memory as they work
//! - Dependency management between tasks
//! - Real-time progress tracking
//! - Collaborative problem-solving with structured workflow
//!
//! The coordinator creates a plan, delegates tasks to specialized agents,
//! and each agent updates the shared memory with their results, allowing
//! other agents to build upon previous work.
//!
//! Run this example with: `cargo run --example forest_with_coordinator`

use helios_engine::{Agent, Config, ForestBuilder};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Forest of Agents with Coordinator-Based Planning\n");

    // Load configuration
    let config = Config::from_file("config.toml")?;

    // Create a Forest of Agents with specialized roles
    let mut forest = ForestBuilder::new()
        .config(config)
        // Coordinator agent - creates plans and manages the team
        .agent(
            "coordinator".to_string(),
            Agent::builder("coordinator")
                .system_prompt(
                    "You are an expert project coordinator and task planner. Your role is to:\n\
                    1. Analyze complex tasks and break them into manageable subtasks\n\
                    2. Create detailed plans using the 'create_plan' tool\n\
                    3. Assign tasks to the most appropriate team members based on their expertise\n\
                    4. Define task dependencies to ensure proper execution order\n\
                    5. Synthesize final results from all team members\n\n\
                    Available team members:\n\
                    - researcher: Gathers information, conducts analysis, finds data\n\
                    - writer: Creates content, documentation, and written materials\n\
                    - analyst: Analyzes data, identifies patterns, provides insights\n\
                    - reviewer: Reviews work quality, provides feedback, ensures standards\n\n\
                    When creating a plan, think carefully about:\n\
                    - What information is needed first (research/data gathering)\n\
                    - What depends on what (task dependencies)\n\
                    - Who is best suited for each task\n\
                    - How to ensure quality (review steps)\n\n\
                    Always use the create_plan tool to structure the work.",
                )
                .max_iterations(20),
        )
        // Research agent - gathers information and data
        .agent(
            "researcher".to_string(),
            Agent::builder("researcher")
                .system_prompt(
                    "You are a research specialist who excels at:\n\
                    - Gathering comprehensive information on topics\n\
                    - Identifying key facts, statistics, and sources\n\
                    - Analyzing information for relevance and accuracy\n\
                    - Providing well-organized research findings\n\n\
                    When completing a task:\n\
                    1. Review the shared memory to see what other agents have done\n\
                    2. Conduct thorough research on your assigned topic\n\
                    3. Use 'update_task_memory' tool to save your findings\n\
                    4. Include key data points that other agents might need\n\n\
                    Be thorough but concise in your responses.",
                )
                .max_iterations(10),
        )
        // Writer agent - creates content
        .agent(
            "writer".to_string(),
            Agent::builder("writer")
                .system_prompt(
                    "You are a skilled content writer who excels at:\n\
                    - Creating clear, engaging, and well-structured content\n\
                    - Adapting tone and style to the audience\n\
                    - Incorporating research and data into narratives\n\
                    - Writing comprehensive yet accessible material\n\n\
                    When completing a task:\n\
                    1. Review the shared memory for research and data from other agents\n\
                    2. Create well-structured content based on the requirements\n\
                    3. Use 'update_task_memory' tool to save your written content\n\
                    4. Ensure your content is complete and ready for review\n\n\
                    Write clearly and professionally.",
                )
                .max_iterations(10),
        )
        // Analyst agent - analyzes data and provides insights
        .agent(
            "analyst".to_string(),
            Agent::builder("analyst")
                .system_prompt(
                    "You are a data analyst who excels at:\n\
                    - Analyzing information and identifying patterns\n\
                    - Drawing insights from data and research\n\
                    - Providing actionable recommendations\n\
                    - Summarizing complex information clearly\n\n\
                    When completing a task:\n\
                    1. Review the shared memory for available data and research\n\
                    2. Analyze the information thoroughly\n\
                    3. Use 'update_task_memory' tool to save your analysis and insights\n\
                    4. Provide clear, actionable conclusions\n\n\
                    Be analytical and data-driven in your responses.",
                )
                .max_iterations(10),
        )
        // Reviewer agent - ensures quality
        .agent(
            "reviewer".to_string(),
            Agent::builder("reviewer")
                .system_prompt(
                    "You are a quality reviewer who excels at:\n\
                    - Reviewing content for accuracy and completeness\n\
                    - Identifying areas for improvement\n\
                    - Ensuring consistency and quality standards\n\
                    - Providing constructive feedback\n\n\
                    When completing a task:\n\
                    1. Review the shared memory to see all completed work\n\
                    2. Evaluate the quality, accuracy, and completeness\n\
                    3. Use 'update_task_memory' tool to save your review and any improvements\n\
                    4. Provide clear assessment and final approval\n\n\
                    Be thorough and constructive in your reviews.",
                )
                .max_iterations(15),
        )
        .max_iterations(30)
        .build()
        .await?;

    println!("✅ Forest created with 5 specialized agents\n");

    let task1 = "Create a comprehensive guide about the benefits of renewable energy. \
                 Include research-backed information, clear explanations, data analysis, \
                 and ensure it's well-reviewed for quality.";

    println!("📋 Task: {}\n", task1);

    let result1 = forest
        .execute_collaborative_task(
            &"coordinator".to_string(),
            task1.to_string(),
            vec![
                "researcher".to_string(),
                "writer".to_string(),
                "analyst".to_string(),
                "reviewer".to_string(),
            ],
        )
        .await?;

    println!("\n{}\n", "=".repeat(70));
    println!("✨ FINAL RESULT:\n{}\n", result1);
    println!("{}\n", "=".repeat(70));

    // Show the shared memory state
    println!("📊 SHARED MEMORY STATE:");
    let context = forest.get_shared_context().await;

    if let Some(plan) = context.get_plan() {
        println!("\n📋 Task Plan Summary:");
        println!("  Objective: {}", plan.objective);
        println!("  Total Tasks: {}", plan.tasks.len());
        println!(
            "  Completed: {}/{}",
            plan.get_progress().0,
            plan.get_progress().1
        );
        println!("\n  Task Breakdown:");
        for task in plan.tasks_in_order() {
            let status_icon = match task.status {
                helios_engine::forest::TaskStatus::Completed => "✅",
                helios_engine::forest::TaskStatus::InProgress => "🔄",
                helios_engine::forest::TaskStatus::Pending => "⏳",
                helios_engine::forest::TaskStatus::Failed => "❌",
            };
            println!(
                "    {} [{}] {} - {}",
                status_icon, task.assigned_to, task.id, task.description
            );
            if let Some(result) = &task.result {
                let preview = if result.len() > 100 {
                    format!("{}...", &result[..100])
                } else {
                    result.clone()
                };
                println!("       Result: {}", preview);
            }
        }
    }

    println!("\n  Shared Data Keys:");
    for key in context.data.keys() {
        if !key.starts_with("current_")
            && !key.starts_with("involved_")
            && !key.starts_with("task_status")
        {
            println!("    • {}", key);
        }
    }

    println!("\n✅ Demo completed successfully!");

    Ok(())
}
