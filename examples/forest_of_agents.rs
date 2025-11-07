//! # Forest of Agents Example
//!
//! This example demonstrates the Forest of Agents feature with STREAMING enabled by default.
//! Watch as multiple agents collaborate in real-time, with their responses streaming
//! token-by-token as they think and communicate.
//!
//! The Forest of Agents enables:
//! - Inter-agent communication and messaging (with streaming responses)
//! - Task delegation between agents
//! - Shared context and memory
//! - Collaborative task execution with real-time output
//!
//! Run this example with: `cargo run --example forest_of_agents`

use helios_engine::{Agent, Config, ForestBuilder};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Helios Engine - Forest of Agents Demo (with Real-Time Streaming)");
    println!("====================================================================\n");
    println!("💡 Note: All agent responses stream in real-time, token by token!\n");

    // Load configuration
    let config = Config::from_file("config.toml")?;

    // Create a Forest of Agents with specialized agents
    // Using the improved syntax to add multiple agents at once!
    let mut forest = ForestBuilder::new()
        .config(config)
        .agents(vec![
            // Coordinator agent - manages the team and delegates tasks
            (
                "coordinator".to_string(),
                Agent::builder("coordinator")
                    .system_prompt(
                        "You are a project coordinator. For simple tasks that you can handle yourself, \
                        complete them directly and provide a complete response. For complex tasks that \
                        require specialized expertise, you can delegate using the 'delegate_task' tool \
                        to agents like 'researcher', 'writer', 'editor', and 'qa'.\n\n\
                        When you delegate a task, WAIT for the response and then synthesize the results. \
                        Always provide a final, complete answer to the user's request."
                    )
                    .max_iterations(10)
            ),
            // Research agent - gathers and analyzes information
            (
                "researcher".to_string(),
                Agent::builder("researcher")
                    .system_prompt(
                        "You are a research specialist who excels at gathering information, \
                        analyzing data, and providing insights. You work closely with the coordinator \
                        and writer to ensure all work is based on accurate information. Use \
                        communication tools to share your findings and request clarification when needed."
                    )
                    .max_iterations(10)
            ),
            // Writer agent - creates content and documentation
            (
                "writer".to_string(),
                Agent::builder("writer")
                    .system_prompt(
                        "You are a skilled writer who creates clear, well-structured content and \
                        documentation. When you receive a task, complete it fully and provide the \
                        final written content. You can use communication tools to request information \
                        from the researcher if needed."
                    )
                    .max_iterations(10)
            ),
            // Editor agent - reviews and improves content
            (
                "editor".to_string(),
                Agent::builder("editor")
                    .system_prompt(
                        "You are an editor who reviews content for quality, clarity, and consistency. \
                        When you receive content to review, provide constructive feedback and an \
                        improved version."
                    )
                    .max_iterations(10)
            ),
            // Quality Assurance agent - validates the final output
            (
                "qa".to_string(),
                Agent::builder("qa")
                    .system_prompt(
                        "You are a quality assurance specialist who validates that all requirements \
                        are met and the output is accurate and complete. When you receive content to \
                        review, verify it meets all requirements and provide your assessment."
                    )
                    .max_iterations(10)
            ),
        ])
        .max_iterations(15)
        .build()
        .await?;

    println!("✅ Created Forest of Agents with 5 specialized agents:");
    println!("  • 🎯 Coordinator: Manages projects and delegates tasks");
    println!("  • 🔬 Researcher: Gathers and analyzes information");
    println!("  • ✍️  Writer: Creates content and documentation");
    println!("  • 📝 Editor: Reviews and improves content quality");
    println!("  • ✅ QA: Validates requirements and final output");
    println!();

    // Demonstrate collaborative task execution with streaming
    println!("🎯 TASK: Create a brief guide on sustainable gardening");
    println!("{}", "=".repeat(70));
    println!();

    println!("🎬 Starting collaborative task execution...");
    println!("   (Watch the responses stream in real-time!)\n");

    // Simpler task for demonstration - DISABLED TOOLS FOR TESTING
    let task = "Create a brief guide (2-3 paragraphs) on sustainable gardening. \
                Include key benefits and one practical technique.";

    println!("📋 Task Description:");
    println!("   {}\n", task);

    println!("{}", "─".repeat(70));
    println!("🤖 COORDINATOR (streaming response):");
    print!("   ");
    io::stdout().flush()?;

    let _result = forest
        .execute_collaborative_task(
            &"coordinator".to_string(),
            task.to_string(),
            vec![
                "researcher".to_string(),
                "writer".to_string(),
                "editor".to_string(),
                "qa".to_string(),
            ],
        )
        .await?;

    println!();
    println!("{}", "─".repeat(70));
    println!();
    println!("✨ Collaborative task completed!");
    println!();

    // Demonstrate direct agent communication with streaming
    println!("💬 Testing direct agent-to-agent communication with streaming:");
    println!("{}", "─".repeat(70));
    println!();

    let mut forest_clone = forest;

    // Test a simple chat to show streaming
    println!("📤 Sending task to Writer agent...");
    println!("🤖 WRITER (streaming response):");
    print!("   ");
    io::stdout().flush()?;

    if let Some(writer) = forest_clone.get_agent_mut(&"writer".to_string()) {
        let _response = writer
            .chat("Write one short paragraph about composting.")
            .await?;
        println!();
    }

    println!();
    println!("{}", "─".repeat(70));
    println!();

    // Send a direct message between agents
    println!("📤 Coordinator → Researcher: Direct message");
    forest_clone
        .send_message(
            &"coordinator".to_string(),
            Some(&"researcher".to_string()),
            "Great job on the research! The information was very helpful.".to_string(),
        )
        .await?;

    forest_clone.process_messages().await?;

    if let Some(researcher) = forest_clone.get_agent(&"researcher".to_string()) {
        let messages = researcher.chat_session().messages.clone();
        if let Some(last_msg) = messages.last() {
            println!("📥 Researcher received: \"{}\"", last_msg.content);
        }
    }
    println!();

    // Demonstrate shared context
    println!("🧠 Shared Context Demo:");
    println!("{}", "─".repeat(70));
    forest_clone
        .set_shared_context(
            "project_status".to_string(),
            serde_json::json!({
                "name": "Sustainable Gardening Guide",
                "status": "completed",
                "contributors": ["coordinator", "researcher", "writer"],
                "completion_date": "2025-11-03"
            }),
        )
        .await;

    let context = forest_clone.get_shared_context().await;
    if let Some(status) = context.get("project_status") {
        println!("📊 Shared project status:");
        println!("{}", serde_json::to_string_pretty(&status).unwrap());
    }
    println!();

    println!("{}", "=".repeat(70));
    println!("✅ Forest of Agents Demo Completed Successfully!");
    println!("{}", "=".repeat(70));
    println!();
    println!("🎉 Key Features Demonstrated:");
    println!("  ✓ Real-time streaming responses from all agents");
    println!("  ✓ Multi-agent collaboration on tasks");
    println!("  ✓ Inter-agent communication (direct messages)");
    println!("  ✓ Task delegation and coordination");
    println!("  ✓ Shared context and memory");
    println!("  ✓ Specialized agent roles working together");
    println!();
    println!("💡 Notice how all responses streamed token-by-token in real-time!");
    println!("   This provides immediate feedback and better user experience.");

    Ok(())
}
