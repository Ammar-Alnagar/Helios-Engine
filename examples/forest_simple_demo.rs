//! Simple Forest of Agents Demo
//!
//! This is a simpler example that demonstrates the planning system
//! with more straightforward prompts and better reliability.

use helios_engine::{Agent, Config, ForestBuilder};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Forest of Agents - Simple Demo\n");

    let config = Config::from_file("config.toml")?;

    // Create a simpler forest with just 3 agents
    let mut forest = ForestBuilder::new()
        .config(config)
        .agent(
            "coordinator".to_string(),
            Agent::builder("coordinator")
                .system_prompt(
                    "You are a project coordinator. Your ONLY job is to create plans, not execute tasks.\n\
                    When given a task, IMMEDIATELY use the create_plan tool with this format:\n\
                    - objective: the overall goal\n\
                    - tasks: JSON array with structure [{\"id\":\"task_1\",\"description\":\"...\",\"assigned_to\":\"worker1\",\"dependencies\":[]}]\n\n\
                    Keep plans simple with 2-3 tasks max. Do NOT try to complete the task yourself."
                )
                .max_iterations(15)
        )
        .agent(
            "worker1".to_string(),
            Agent::builder("worker1")
                .system_prompt(
                    "You are a helpful worker. Complete the task assigned to you and use the \
                    update_task_memory tool to save your results. Be brief and direct."
                )
                .max_iterations(8)
        )
        .agent(
            "worker2".to_string(),
            Agent::builder("worker2")
                .system_prompt(
                    "You are a helpful worker. Complete the task assigned to you and use the \
                    update_task_memory tool to save your results. Be brief and direct."
                )
                .max_iterations(8)
        )
        .max_iterations(20)
        .build()
        .await?;

    println!("✅ Forest created with 3 agents\n");

    // Simple task
    let task = "List 3 benefits of exercise. Keep it brief.";
    println!("📋 Task: {}\n", task);

    let result = forest
        .execute_collaborative_task(
            &"coordinator".to_string(),
            task.to_string(),
            vec!["worker1".to_string(), "worker2".to_string()],
        )
        .await?;

    println!("\n{}\n", "=".repeat(60));
    println!("✨ RESULT:\n{}\n", result);
    println!("{}\n", "=".repeat(60));

    // Show task breakdown
    let context = forest.get_shared_context().await;
    if let Some(plan) = context.get_plan() {
        println!("📊 Plan Summary:");
        let (completed, total) = plan.get_progress();
        println!("  Completed: {}/{} tasks\n", completed, total);

        for task in plan.tasks_in_order() {
            let status = match task.status {
                helios_engine::forest::TaskStatus::Completed => "✅",
                helios_engine::forest::TaskStatus::InProgress => "🔄",
                helios_engine::forest::TaskStatus::Pending => "⏳",
                helios_engine::forest::TaskStatus::Failed => "❌",
            };
            println!("  {} [{}] {}", status, task.assigned_to, task.description);
        }
    } else {
        println!("📊 No plan was created (coordinator handled directly)");
    }

    println!("\n✅ Demo completed!");

    Ok(())
}
