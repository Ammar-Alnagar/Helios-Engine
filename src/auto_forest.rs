//! # AutoForest Module
//!
//! This module implements automatic orchestration of a forest of agents.
//! Given a high-level task, the AutoForest orchestrator intelligently:
//! - Determines the optimal number of agents to spawn
//! - Generates specialized prompts for each agent
//! - Distributes available tools among agents
//! - Coordinates task execution and result aggregation
//!
//! # Example
//!
//! ```rust,no_run
//! use helios_engine::{AutoForest, Config, CalculatorTool};
//!
//! #[tokio::main]
//! async fn main() -> helios_engine::Result<()> {
//!     let config = Config::new_default();
//!     
//!     let mut auto_forest = AutoForest::new(config)
//!         .with_tools(vec![Box::new(CalculatorTool)])
//!         .build()
//!         .await?;
//!
//!     let task = "Analyze sales data and identify trends";
//!     let result = auto_forest.execute_task(task).await?;
//!     println!("Result: {}", result);
//!     Ok(())
//! }
//! ```

use crate::agent::Agent;
use crate::config::Config;
use crate::error::{HeliosError, Result};
use crate::tools::Tool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for an agent spawned by the orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Name of the agent
    pub name: String,
    /// Specialized system prompt for this agent
    pub system_prompt: String,
    /// Indices of tools this agent has access to
    #[serde(default)]
    pub tool_indices: Vec<usize>,
    /// Role/specialty of this agent
    pub role: String,
}

/// Internal struct for deserializing the orchestration plan from LLM response
#[derive(Debug, Deserialize)]
struct OrchestrationPlanJson {
    num_agents: usize,
    reasoning: String,
    agents: Vec<AgentConfig>,
    task_breakdown: HashMap<String, String>,
}

/// An auto-spawned agent with its assigned configuration
pub struct SpawnedAgent {
    /// The agent instance
    pub agent: Agent,
    /// Configuration for this agent
    pub config: AgentConfig,
    /// Result from this agent's execution
    pub result: Option<String>,
}

/// Orchestration plan generated for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationPlan {
    /// Overall task description
    pub task: String,
    /// Number of agents to spawn
    pub num_agents: usize,
    /// Reasoning for the chosen configuration
    pub reasoning: String,
    /// Configurations for each agent
    pub agents: Vec<AgentConfig>,
    /// Task breakdown for each agent
    pub task_breakdown: HashMap<String, String>,
}

/// The AutoForest orchestrator - manages automatic agent spawning and coordination
pub struct AutoForest {
    config: Config,
    tools: Vec<Box<dyn Tool>>,
    spawned_agents: Vec<SpawnedAgent>,
    orchestration_plan: Option<OrchestrationPlan>,
    orchestrator_agent: Option<Agent>,
}

impl AutoForest {
    /// Creates a new AutoForest orchestrator builder
    #[allow(clippy::new_ret_no_self)]
    pub fn new(config: Config) -> AutoForestBuilder {
        AutoForestBuilder::new(config)
    }

    /// Gets the current orchestration plan
    pub fn orchestration_plan(&self) -> Option<&OrchestrationPlan> {
        self.orchestration_plan.as_ref()
    }

    /// Gets the spawned agents
    pub fn spawned_agents(&self) -> &[SpawnedAgent] {
        &self.spawned_agents
    }

    /// Generates an orchestration plan for the given task
    async fn generate_orchestration_plan(&mut self, task: &str) -> Result<OrchestrationPlan> {
        // Create a system prompt for the orchestrator
        let tools_info = self
            .tools
            .iter()
            .enumerate()
            .map(|(i, tool)| format!("- Tool {}: {} ({})", i, tool.name(), tool.description()))
            .collect::<Vec<_>>()
            .join("\n");

        let orchestrator_prompt = format!(
            r#"You are an expert task orchestrator. Your job is to analyze a task and create an optimal plan for a forest of AI agents to complete it.

Available tools:
{}

Given the task, you must:
1. Determine the optimal number of agents (1-5)
2. Define each agent's role and specialization
3. Create specialized system prompts for each agent
4. Assign tools to each agent based on their role
5. Break down the task into subtasks for each agent

Respond with ONLY a JSON object with this structure (no markdown, no extra text):
{{
  "num_agents": <number>,
  "reasoning": "<brief explanation>",
  "agents": [
    {{
      "name": "<agent_name>",
      "role": "<role>",
      "system_prompt": "<specialized_prompt>",
      "tool_indices": [<indices>]
    }}
  ],
  "task_breakdown": {{
    "<agent_name>": "<specific_task_for_this_agent>"
  }}
}}"#,
            tools_info
        );

        // Create orchestrator agent if not exists
        if self.orchestrator_agent.is_none() {
            let orchestrator = Agent::builder("Orchestrator")
                .config(self.config.clone())
                .system_prompt(&orchestrator_prompt)
                .build()
                .await?;
            self.orchestrator_agent = Some(orchestrator);
        }

        // Get the orchestrator agent
        let orchestrator = self.orchestrator_agent.as_mut().ok_or_else(|| {
            HeliosError::AgentError("Failed to create orchestrator agent".to_string())
        })?;

        // Ask orchestrator to generate plan
        let response = orchestrator.chat(&format!("Task: {}", task)).await?;

        // Parse the response as JSON using serde for type-safe deserialization
        let plan_data: OrchestrationPlanJson = serde_json::from_str(&response).map_err(|e| {
            HeliosError::AgentError(format!("Failed to parse orchestration plan: {}", e))
        })?;

        // Construct the orchestration plan from the parsed data
        let plan = OrchestrationPlan {
            task: task.to_string(),
            num_agents: plan_data.num_agents,
            reasoning: plan_data.reasoning,
            agents: plan_data.agents,
            task_breakdown: plan_data.task_breakdown,
        };

        self.orchestration_plan = Some(plan.clone());
        Ok(plan)
    }

    /// Spawns agents according to the orchestration plan
    async fn spawn_agents_from_plan(&mut self, plan: &OrchestrationPlan) -> Result<()> {
        self.spawned_agents.clear();

        for agent_config in &plan.agents {
            // Create the agent
            let agent = Agent::builder(&agent_config.name)
                .config(self.config.clone())
                .system_prompt(&agent_config.system_prompt)
                .build()
                .await?;

            // Note: Tools would be assigned here if we had a mechanism to clone tools
            // For now, agents are created without tools and rely on the LLM's capabilities

            let spawned = SpawnedAgent {
                agent,
                config: agent_config.clone(),
                result: None,
            };

            self.spawned_agents.push(spawned);
        }

        Ok(())
    }

    /// Executes a task using the auto-forest orchestration
    pub async fn execute_task(&mut self, task: &str) -> Result<String> {
        // Generate orchestration plan
        let plan = self.generate_orchestration_plan(task).await?;

        // Spawn agents according to plan
        self.spawn_agents_from_plan(&plan).await?;

        // Execute tasks on spawned agents IN PARALLEL using tokio::join_all
        let mut futures = Vec::new();

        for spawned_agent in self.spawned_agents.drain(..) {
            let agent_task = plan
                .task_breakdown
                .get(&spawned_agent.config.name)
                .cloned()
                .unwrap_or_else(|| format!("Complete your assigned portion of: {}", task));

            let future = async move {
                let mut agent = spawned_agent.agent;
                let config = spawned_agent.config;
                let result = agent.chat(&agent_task).await;
                (agent, config, result)
            };

            futures.push(future);
        }

        // Wait for all agents to complete in parallel
        let completed_agents = futures::future::join_all(futures).await;

        // Collect results and restore agents
        let mut results = HashMap::new();
        self.spawned_agents.clear(); // Ensure vector is empty before repopulating

        for (agent, config, result) in completed_agents {
            let agent_name = config.name.clone();
            let (result_string, result_for_map) = match result {
                Ok(output) => (Some(output.clone()), output),
                Err(e) => {
                    let err_msg = format!("Error: {}", e);
                    (Some(err_msg.clone()), err_msg)
                }
            };
            results.insert(agent_name, result_for_map);

            self.spawned_agents.push(SpawnedAgent {
                agent,
                config,
                result: result_string,
            });
        }

        // Aggregate results
        let aggregated_result = self.aggregate_results(&results, task).await?;

        Ok(aggregated_result)
    }

    /// Shorthand: execute a task with just one method call
    pub async fn do_task(&mut self, task: &str) -> Result<String> {
        self.execute_task(task).await
    }

    /// Ultra-simple: shorthand for asking the forest to complete a task
    pub async fn run(&mut self, task: &str) -> Result<String> {
        self.execute_task(task).await
    }

    /// Aggregates results from all agents into a final response
    async fn aggregate_results(
        &mut self,
        results: &HashMap<String, String>,
        task: &str,
    ) -> Result<String> {
        let mut result_text = String::new();
        result_text.push_str("## Task Execution Summary\n\n");
        result_text.push_str(&format!("**Task**: {}\n\n", task));
        result_text.push_str("### Agent Results:\n\n");

        for (agent_name, result) in results {
            result_text.push_str(&format!("**{}**:\n{}\n\n", agent_name, result));
        }

        // Use orchestrator to synthesize final answer if multiple agents
        if results.len() > 1 {
            result_text.push_str("### Synthesized Analysis:\n\n");
            let orchestrator = self
                .orchestrator_agent
                .as_mut()
                .ok_or_else(|| HeliosError::AgentError("Orchestrator not available".to_string()))?;

            let synthesis_prompt = format!(
                "Synthesize these agent results into a cohesive answer:\n{}",
                result_text
            );
            let synthesis = orchestrator.chat(&synthesis_prompt).await?;
            result_text.push_str(&synthesis);
        }

        Ok(result_text)
    }
}

/// Builder for AutoForest
pub struct AutoForestBuilder {
    config: Config,
    tools: Vec<Box<dyn Tool>>,
}

impl AutoForestBuilder {
    /// Creates a new AutoForestBuilder with the given config
    pub fn new(config: Config) -> Self {
        Self {
            config,
            tools: Vec::new(),
        }
    }

    /// Sets the tools available to the forest
    pub fn with_tools(mut self, tools: Vec<Box<dyn Tool>>) -> Self {
        self.tools = tools;
        self
    }

    /// Builds the AutoForest orchestrator
    pub async fn build(self) -> Result<AutoForest> {
        Ok(AutoForest {
            config: self.config,
            tools: self.tools,
            spawned_agents: Vec::new(),
            orchestration_plan: None,
            orchestrator_agent: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_creation() {
        let config = AgentConfig {
            name: "TestAgent".to_string(),
            system_prompt: "You are helpful".to_string(),
            tool_indices: vec![0, 1],
            role: "Analyzer".to_string(),
        };

        assert_eq!(config.name, "TestAgent");
        assert_eq!(config.tool_indices.len(), 2);
    }

    #[test]
    fn test_orchestration_plan_creation() {
        let plan = OrchestrationPlan {
            task: "Test task".to_string(),
            num_agents: 2,
            reasoning: "Two agents needed".to_string(),
            agents: vec![],
            task_breakdown: HashMap::new(),
        };

        assert_eq!(plan.num_agents, 2);
        assert_eq!(plan.task, "Test task");
    }
}
