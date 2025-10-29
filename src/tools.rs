use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use crate::error::{HeliosError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: ParametersSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParametersSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: HashMap<String, ToolParameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
}

impl ToolResult {
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            output: message.into(),
        }
    }
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> HashMap<String, ToolParameter>;
    async fn execute(&self, args: Value) -> Result<ToolResult>;

    fn to_definition(&self) -> ToolDefinition {
        let required: Vec<String> = self
            .parameters()
            .iter()
            .filter(|(_, param)| param.required.unwrap_or(false))
            .map(|(name, _)| name.clone())
            .collect();

        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: self.name().to_string(),
                description: self.description().to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: self.parameters(),
                    required: if required.is_empty() { None } else { Some(required) },
                },
            },
        }
    }
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|b| &**b)
    }

    pub async fn execute(&self, name: &str, args: Value) -> Result<ToolResult> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| HeliosError::ToolError(format!("Tool '{}' not found", name)))?;
        
        tool.execute(args).await
    }

    pub fn get_definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|tool| tool.to_definition())
            .collect()
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Example built-in tools

pub struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Perform basic arithmetic operations. Supports +, -, *, / operations."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "expression".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Mathematical expression to evaluate (e.g., '2 + 2')".to_string(),
                required: Some(true),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let expression = args
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'expression' parameter".to_string()))?;

        // Simple expression evaluator
        let result = evaluate_expression(expression)?;
        Ok(ToolResult::success(result.to_string()))
    }
}

fn evaluate_expression(expr: &str) -> Result<f64> {
    let expr = expr.replace(" ", "");
    
    // Simple parsing for basic operations
    for op in &['*', '/', '+', '-'] {
        if let Some(pos) = expr.rfind(*op) {
            if pos == 0 {
                continue; // Skip if operator is at the beginning (negative number)
            }
            let left = &expr[..pos];
            let right = &expr[pos + 1..];
            
            let left_val = evaluate_expression(left)?;
            let right_val = evaluate_expression(right)?;
            
            return Ok(match op {
                '+' => left_val + right_val,
                '-' => left_val - right_val,
                '*' => left_val * right_val,
                '/' => {
                    if right_val == 0.0 {
                        return Err(HeliosError::ToolError("Division by zero".to_string()));
                    }
                    left_val / right_val
                }
                _ => unreachable!(),
            });
        }
    }
    
    expr.parse::<f64>()
        .map_err(|_| HeliosError::ToolError(format!("Invalid expression: {}", expr)))
}

pub struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echo back the provided message."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "message".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "The message to echo back".to_string(),
                required: Some(true),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let message = args
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'message' parameter".to_string()))?;

        Ok(ToolResult::success(format!("Echo: {}", message)))
    }
}
