//! # Forest of Agents Example
//!
//! This example demonstrates the Forest of Agents feature, which allows multiple agents
//! to collaborate, communicate, and share context to accomplish complex tasks together.
//!
//! The Forest of Agents enables:
//! - Inter-agent communication and messaging
//! - Task delegation between agents
//! - Shared context and memory
//! - Collaborative task execution
//!
//! Run this example with: `cargo run --example forest_of_agents`

use helios_engine::{Agent, Config, ForestBuilder};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Helios Engine - Forest of Agents Demo");
    println!("=========================================\n");

    // Load configuration
    let config = Config::from_file("config.toml")?;

    // Create a Forest of Agents with specialized agents
    // You can add as many agents as you want!
    let mut forest = ForestBuilder::new()
        .config(config)
        // Coordinator agent - manages the team and delegates tasks
        .agent(
            "coordinator".to_string(),
            Agent::builder("coordinator")
                .system_prompt(
                    "You are a project coordinator responsible for breaking down complex tasks \
                    and delegating them to specialized team members. You communicate with other \
                    agents to ensure the project is completed successfully. Use the available \
                    communication tools to delegate tasks, share information, and coordinate work."
                )
        )
        // Research agent - gathers and analyzes information
        .agent(
            "researcher".to_string(),
            Agent::builder("researcher")
                .system_prompt(
                    "You are a research specialist who excels at gathering information, \
                    analyzing data, and providing insights. You work closely with the coordinator \
                    and writer to ensure all work is based on accurate information. Use \
                    communication tools to share your findings and request clarification when needed."
                )
        )
        // Writer agent - creates content and documentation
        .agent(
            "writer".to_string(),
            Agent::builder("writer")
                .system_prompt(
                    "You are a skilled writer who creates clear, well-structured content and \
                    documentation. You work with the coordinator and researcher to produce \
                    high-quality written materials. Use communication tools to request information \
                    from the researcher and coordinate with the coordinator on project requirements."
                )
        )
        // Editor agent - reviews and improves content
        .agent(
            "editor".to_string(),
            Agent::builder("editor")
                .system_prompt(
                    "You are an editor who reviews content for quality, clarity, and consistency. \
                    You provide feedback to the writer and ensure the final product meets high \
                    standards. Use communication tools to request revisions and share feedback."
                )
        )
        // Quality Assurance agent - validates the final output
        .agent(
            "qa".to_string(),
            Agent::builder("qa")
                .system_prompt(
                    "You are a quality assurance specialist who validates that all requirements \
                    are met and the output is accurate and complete. You work with all team members \
                    to ensure the final deliverable is of the highest quality."
                )
        )
        .max_iterations(5)
        .build()
        .await?;

    println!("✓ Created Forest of Agents with 5 specialized agents:");
    println!("  • Coordinator: Manages projects and delegates tasks");
    println!("  • Researcher: Gathers and analyzes information");
    println!("  • Writer: Creates content and documentation");
    println!("  • Editor: Reviews and improves content quality");
    println!("  • QA: Validates requirements and final output");
    println!();

    // Demonstrate collaborative task execution
    println!("🎯 Executing collaborative task:");
    println!("\"Create a comprehensive guide on sustainable gardening practices\"");
    println!();

    let result = forest
        .execute_collaborative_task(
            &"coordinator".to_string(),
            "Create a comprehensive guide on sustainable gardening practices. This should include \
            environmental benefits, practical techniques, common challenges, and tips for beginners. \
            Make it informative yet accessible to people new to sustainable gardening.".to_string(),
            vec![
                "researcher".to_string(),
                "writer".to_string(),
                "editor".to_string(),
                "qa".to_string(),
            ],
        )
        .await?;

    println!("📄 Final Result:");
    println!("{}", "=".repeat(60));
    println!("{}", result);
    println!("{}", "=".repeat(60));
    println!();

    // Demonstrate direct agent communication
    println!("💬 Demonstrating inter-agent communication:");
    println!();

    let mut forest_clone = forest; // Clone for mutable operations

    // Send a direct message
    println!("📤 Coordinator sends a message to Researcher...");
    forest_clone
        .send_message(
            &"coordinator".to_string(),
            Some(&"researcher".to_string()),
            "Please research the latest sustainable gardening techniques for urban environments."
                .to_string(),
        )
        .await?;

    // Process messages
    forest_clone.process_messages().await?;

    // Check what the researcher received
    if let Some(researcher) = forest_clone.get_agent(&"researcher".to_string()) {
        let messages = researcher.chat_session().messages.clone();
        if let Some(last_msg) = messages.last() {
            println!("📥 Researcher received: {}", last_msg.content);
        }
    }

    // Send a broadcast message
    println!("\n📢 Coordinator broadcasts an update...");
    forest_clone
        .send_message(
            &"coordinator".to_string(),
            None, // None = broadcast
            "Team update: We've successfully completed the sustainable gardening guide. Great collaboration everyone!".to_string(),
        )
        .await?;

    forest_clone.process_messages().await?;

    // Check what agents received
    for agent_id in ["coordinator", "researcher", "writer", "editor", "qa"] {
        if let Some(agent) = forest_clone.get_agent(&agent_id.to_string()) {
            let messages = agent.chat_session().messages.clone();
            if let Some(last_msg) = messages.last() {
                if last_msg.content.contains("broadcast") {
                    println!("📥 {} received broadcast: {}", agent_id, last_msg.content);
                }
            }
        }
    }

    // Demonstrate shared context
    println!("\n🧠 Demonstrating shared context:");
    forest_clone
        .set_shared_context(
            "project_status".to_string(),
            serde_json::json!({
                "name": "Sustainable Gardening Guide",
                "status": "completed",
                "contributors": ["coordinator", "researcher", "writer"],
                "completion_date": "2025-11-02"
            }),
        )
        .await;

    let context = forest_clone.get_shared_context().await;
    if let Some(status) = context.get("project_status") {
        println!("📊 Shared project status: {}", status);
    }

    println!("\n✅ Forest of Agents demo completed successfully!");
    println!("\nKey features demonstrated:");
    println!("  • Multi-agent collaboration on complex tasks");
    println!("  • Inter-agent communication (direct and broadcast)");
    println!("  • Task delegation and coordination");
    println!("  • Shared context and memory");
    println!("  • Specialized agent roles working together");

    Ok(())
}
