//! # Forest of Agents with File System Analysis
//!
//! This example demonstrates the Forest of Agents using file tools to analyze
//! a project directory structure. The coordinator creates a plan, and specialized
//! agents use file system tools to research and analyze the codebase.
//!
//! The workflow:
//! 1. Coordinator creates analysis plan
//! 2. File Explorer agent maps the directory structure
//! 3. Code Analyzer agent examines source files
//! 4. Documentation Analyst reviews docs
//! 5. Report Writer creates comprehensive analysis
//!
//! Run this example with: `cargo run --example forest_with_file_analysis`

use helios_engine::{Agent, Config, FileListTool, FileReadTool, FileSearchTool, ForestBuilder};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    println!("🚀 Forest of Agents - File System Analysis\n");

    // Load configuration
    let config = Config::from_file("config.toml")?;

    // Create a Forest of Agents with specialized roles and file tools
    let mut forest = ForestBuilder::new()
        .config(config)
        // Coordinator agent - creates analysis plan
        .agent(
            "coordinator".to_string(),
            Agent::builder("coordinator")
                .system_prompt(
                    "You are a project analysis coordinator. Your role is to:\n\
                    1. Create a structured plan to analyze a project directory\n\
                    2. Break down the analysis into specific tasks for specialized agents\n\
                    3. Assign tasks based on agent expertise\n\
                    4. Synthesize findings into a comprehensive report\n\n\
                    Available team members:\n\
                    - file_explorer: Maps directory structure, lists files and folders\n\
                    - code_analyzer: Analyzes source code files, identifies patterns\n\
                    - doc_analyst: Reviews documentation files (README, docs)\n\
                    - report_writer: Creates comprehensive analysis reports\n\n\
                    Create a plan that:\n\
                    - First explores the directory structure\n\
                    - Then analyzes code files\n\
                    - Reviews documentation\n\
                    - Finally synthesizes findings\n\n\
                    Use the create_plan tool to structure the work with proper dependencies.",
                )
                .max_iterations(15),
        )
        // File Explorer agent - explores directory structure
        .agent(
            "file_explorer".to_string(),
            Agent::builder("file_explorer")
                .system_prompt(
                    "You are a file system explorer specialist. Your expertise:\n\
                    - Mapping directory structures\n\
                    - Listing files and folders\n\
                    - Identifying file types and organization patterns\n\
                    - Understanding project layouts\n\n\
                    Tools available:\n\
                    - file_list: List files in directories\n\
                    - file_read: Read file contents (for small files)\n\
                    - file_search: Search for files matching patterns\n\n\
                    When completing tasks:\n\
                    1. Use file_list to explore directory structure\n\
                    2. Identify key directories (src, docs, examples, tests, etc.)\n\
                    3. Count files by type (.rs, .md, .toml, etc.)\n\
                    4. Use update_task_memory to save your findings\n\
                    5. Include directory tree and file statistics\n\n\
                    Be thorough but organized in your exploration.",
                )
                .tool(Box::new(FileListTool))
                .tool(Box::new(FileReadTool))
                .tool(Box::new(FileSearchTool))
                .max_iterations(12),
        )
        // Code Analyzer agent - analyzes source code
        .agent(
            "code_analyzer".to_string(),
            Agent::builder("code_analyzer")
                .system_prompt(
                    "You are a code analysis specialist. Your expertise:\n\
                    - Analyzing source code structure\n\
                    - Identifying modules and components\n\
                    - Understanding code organization\n\
                    - Recognizing patterns and architecture\n\n\
                    Tools available:\n\
                    - file_list: List code files\n\
                    - file_read: Read source files\n\
                    - file_search: Find specific code patterns\n\n\
                    When completing tasks:\n\
                    1. Review shared memory for directory structure from file_explorer\n\
                    2. Use file_read to examine key source files (lib.rs, main.rs, etc.)\n\
                    3. Identify main modules and their purposes\n\
                    4. Look for important structs, traits, and functions\n\
                    5. Use update_task_memory to save your analysis\n\
                    6. Include code structure and key components\n\n\
                    Focus on understanding the architecture and main components.",
                )
                .tool(Box::new(FileListTool))
                .tool(Box::new(FileReadTool))
                .tool(Box::new(FileSearchTool))
                .max_iterations(12),
        )
        // Documentation Analyst agent - reviews documentation
        .agent(
            "doc_analyst".to_string(),
            Agent::builder("doc_analyst")
                .system_prompt(
                    "You are a documentation analysis specialist. Your expertise:\n\
                    - Reviewing README files and documentation\n\
                    - Analyzing project descriptions\n\
                    - Understanding feature documentation\n\
                    - Evaluating documentation completeness\n\n\
                    Tools available:\n\
                    - file_list: List documentation files\n\
                    - file_read: Read documentation content\n\
                    - file_search: Find specific documentation\n\n\
                    When completing tasks:\n\
                    1. Review shared memory for project structure\n\
                    2. Use file_read to examine README.md and docs/\n\
                    3. Identify documented features and capabilities\n\
                    4. Note any examples or guides available\n\
                    5. Use update_task_memory to save your findings\n\
                    6. Include documentation summary and key features\n\n\
                    Be thorough in identifying what the project does and how it's documented.",
                )
                .tool(Box::new(FileListTool))
                .tool(Box::new(FileReadTool))
                .tool(Box::new(FileSearchTool))
                .max_iterations(12),
        )
        // Report Writer agent - synthesizes findings
        .agent(
            "report_writer".to_string(),
            Agent::builder("report_writer")
                .system_prompt(
                    "You are a technical report writer. Your expertise:\n\
                    - Synthesizing technical information\n\
                    - Creating comprehensive analysis reports\n\
                    - Organizing findings clearly\n\
                    - Writing executive summaries\n\n\
                    When completing tasks:\n\
                    1. Review ALL data in shared memory from other agents\n\
                    2. Synthesize findings from file explorer, code analyzer, and doc analyst\n\
                    3. Create a well-structured report with sections:\n\
                       - Project Overview\n\
                       - Directory Structure\n\
                       - Code Architecture\n\
                       - Key Features\n\
                       - Documentation Quality\n\
                       - Summary\n\
                    4. Use update_task_memory to save your report\n\
                    5. Make it comprehensive but readable\n\n\
                    Your report should give a complete picture of the project.",
                )
                .max_iterations(10),
        )
        .max_iterations(25)
        .build()
        .await?;

    println!("✅ Forest created with 5 specialized agents\n");

    let analysis_task = "Perform a comprehensive analysis of this Rust project. \
                        I want to understand:\n\
                        1. The directory structure and organization\n\
                        2. What the main code modules are and what they do\n\
                        3. What features are documented in README and docs\n\
                        4. Overall project architecture and purpose\n\n\
                        Create a detailed analysis report with all findings.";

    println!("📋 Analysis Task:\n{}\n", analysis_task);

    let result = forest
        .execute_collaborative_task(
            &"coordinator".to_string(),
            analysis_task.to_string(),
            vec![
                "file_explorer".to_string(),
                "code_analyzer".to_string(),
                "doc_analyst".to_string(),
                "report_writer".to_string(),
            ],
        )
        .await?;

    println!("\n{}\n", "=".repeat(70));
    println!("📊 ANALYSIS REPORT:\n{}\n", result);
    println!("{}\n", "=".repeat(70));

    // Show detailed task breakdown
    println!("🔍 TASK EXECUTION DETAILS:");

    let context = forest.get_shared_context().await;

    if let Some(plan) = context.get_plan() {
        println!("\n📋 Analysis Plan:");
        println!("  Objective: {}", plan.objective);
        let (completed, total) = plan.get_progress();
        println!("  Progress: {}/{} tasks completed\n", completed, total);

        println!("  Task Breakdown:");
        for (idx, task) in plan.tasks_in_order().iter().enumerate() {
            let status_icon = match task.status {
                helios_engine::forest::TaskStatus::Completed => "✅",
                helios_engine::forest::TaskStatus::InProgress => "🔄",
                helios_engine::forest::TaskStatus::Pending => "⏳",
                helios_engine::forest::TaskStatus::Failed => "❌",
            };

            println!(
                "\n  {}. {} [{}] {}",
                idx + 1,
                status_icon,
                task.assigned_to,
                task.id
            );
            println!("     Task: {}", task.description);

            if !task.dependencies.is_empty() {
                println!("     Dependencies: {}", task.dependencies.join(", "));
            }

            if let Some(result) = &task.result {
                let preview = if result.len() > 150 {
                    format!("{}...", &result[..150])
                } else {
                    result.clone()
                };
                println!("     Result: {}", preview);
            }
        }
    }

    // Show shared data keys
    println!("\n\n📊 Shared Memory Data:");
    let mut data_keys: Vec<_> = context
        .data
        .keys()
        .filter(|k| k.starts_with("task_data_"))
        .collect();
    data_keys.sort();

    if data_keys.is_empty() {
        println!("  (No additional shared data stored)");
    } else {
        for key in data_keys {
            println!("  • {}", key);
        }
    }

    println!("\n✅ Analysis completed successfully!");

    Ok(())
}
