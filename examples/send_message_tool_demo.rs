//! # SendMessageTool Demo Example
//!
//! This example demonstrates the SendMessageTool functionality, which allows agents
//! to communicate within a Forest of Agents. It tests both direct messaging and
//! broadcast messaging capabilities.
//!
//! The SendMessageTool enables:
//! - Direct messaging between specific agents
//! - Broadcast messaging to all agents in the forest
//! - Integration with shared context and message history
//!
//! Run this example with: `cargo run --example send_message_tool_demo`

use helios_engine::{Agent, Config, ForestBuilder, SendMessageTool, Tool};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("📨 Helios Engine - SendMessageTool Demo");
    println!("=======================================\n");

    // Load configuration
    let config = Config::from_file("config.toml")?;
    println!("✓ Loaded configuration from config.toml");

    // Create a simple forest with two agents
    let mut forest = ForestBuilder::new()
        .config(config)
        .agent(
            "alice".to_string(),
            Agent::builder("alice")
                .system_prompt("You are Alice, a helpful communication assistant."),
        )
        .agent(
            "bob".to_string(),
            Agent::builder("bob")
                .system_prompt("You are Bob, a friendly colleague who responds to messages."),
        )
        .max_iterations(3)
        .build()
        .await?;

    println!("✓ Created Forest with 2 agents: Alice and Bob");
    println!();

    // Demonstrate SendMessageTool direct messaging
    println!("📤 Testing SendMessageTool - Direct Message:");
    println!("---------------------------------------------");

    // Create the tool for Alice
    let message_queue = Arc::new(RwLock::new(Vec::new()));
    let shared_context = Arc::new(RwLock::new(helios_engine::SharedContext::new()));

    let send_tool = SendMessageTool::new(
        "alice".to_string(),
        Arc::clone(&message_queue),
        Arc::clone(&shared_context),
    );

    // Test 1: Send a direct message from Alice to Bob
    println!("1. Alice sends a direct message to Bob...");

    let direct_message_args = serde_json::json!({
        "to": "bob",
        "message": "Hi Bob! How are you doing today?"
    });

    let result = send_tool.execute(direct_message_args).await?;
    println!("   Tool result: {}", result.output);
    println!("   Success: {}", result.success);

    // Check the message queue
    {
        let queue = message_queue.read().await;
        println!("   Messages in queue: {}", queue.len());
        if let Some(msg) = queue.first() {
            println!("   Message details:");
            println!("     From: {}", msg.from);
            println!("     To: {:?}", msg.to);
            println!("     Content: {}", msg.content);
        }
    }

    // Check shared context
    {
        let context = shared_context.read().await;
        let messages = context.get_recent_messages(10);
        println!("   Messages in shared context: {}", messages.len());
    }

    println!();

    // Test 2: Send a broadcast message
    println!("2. Alice broadcasts a message to everyone...");

    let broadcast_message_args = serde_json::json!({
        "message": "Hello everyone! This is a broadcast message from Alice."
    });

    let result2 = send_tool.execute(broadcast_message_args).await?;
    println!("   Tool result: {}", result2.output);
    println!("   Success: {}", result2.success);

    // Check the message queue after broadcast
    {
        let queue = message_queue.read().await;
        println!("   Messages in queue: {}", queue.len());
        if let Some(msg) = queue.last() {
            println!("   Latest message details:");
            println!("     From: {}", msg.from);
            println!("     To: {:?}", msg.to);
            println!("     Content: {}", msg.content);
        }
    }

    println!();

    // Demonstrate integration with Forest messaging system
    println!("🌲 Testing Forest Integration:");
    println!("------------------------------");

    // Clear our test queues and use the forest's messaging system
    {
        let mut queue = message_queue.write().await;
        queue.clear();
    }

    println!("3. Using Forest's messaging system...");

    // Send message through the forest
    forest
        .send_message(
            &"alice".to_string(),
            Some(&"bob".to_string()),
            "Hello Bob via Forest messaging!".to_string(),
        )
        .await?;

    // Process messages
    forest.process_messages().await?;

    // Check if Bob received the message
    if let Some(bob) = forest.get_agent(&"bob".to_string()) {
        let messages = bob.chat_session().messages.clone();
        println!("   Bob's message count: {}", messages.len());
        if let Some(last_msg) = messages.last() {
            println!("   Bob received: {}", last_msg.content);
        }
    }

    println!();

    // Test broadcast through forest
    println!("4. Forest broadcast message...");

    forest
        .send_message(
            &"alice".to_string(),
            None, // Broadcast
            "Forest broadcast: Meeting at 3 PM!".to_string(),
        )
        .await?;

    forest.process_messages().await?;

    // Check all agents received the broadcast
    for agent_id in ["alice", "bob"] {
        if let Some(agent) = forest.get_agent(&agent_id.to_string()) {
            let messages = agent.chat_session().messages.clone();
            if let Some(last_msg) = messages.last() {
                if last_msg.content.contains("broadcast") || last_msg.content.contains("Meeting") {
                    println!("   {} received broadcast: {}", agent_id, last_msg.content);
                }
            }
        }
    }

    println!();
    println!("✅ SendMessageTool demo completed successfully!");
    println!();
    println!("Key features tested:");
    println!("  • Direct messaging between agents");
    println!("  • Broadcast messaging to all agents");
    println!("  • Message queue management");
    println!("  • Shared context integration");
    println!("  • Forest messaging system integration");

    Ok(())
}
