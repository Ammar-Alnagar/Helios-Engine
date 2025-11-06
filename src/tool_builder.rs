//! # Tool Builder Module
//!
//! This module provides a simplified way to create tools without implementing the Tool trait manually.
//! Users can create tools by providing a name, description, parameters, and a function to execute.

use crate::error::{HeliosError, Result};
use crate::tools::{Tool, ToolParameter, ToolResult};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Type alias for the tool execution function.
/// This is a boxed async function that takes JSON arguments and returns a ToolResult.
pub type ToolFunction =
    Arc<dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<ToolResult>> + Send>> + Send + Sync>;

/// A builder for creating tools with a simplified API.
///
/// # Example
///
/// ```rust
/// use helios_engine::ToolBuilder;
/// use serde_json::Value;
///
/// async fn my_calculator(args: Value) -> helios_engine::Result<helios_engine::ToolResult> {
///     let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
///     let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
///     Ok(helios_engine::ToolResult::success((a + b).to_string()))
/// }
///
/// # async fn example() -> helios_engine::Result<()> {
/// let tool = ToolBuilder::new("add_numbers")
///     .description("Add two numbers together")
///     .parameter("a", "number", "First number", true)
///     .parameter("b", "number", "Second number", true)
///     .function(my_calculator)
///     .build();
/// # Ok(())
/// # }
/// ```
pub struct ToolBuilder {
    name: String,
    description: String,
    parameters: HashMap<String, ToolParameter>,
    function: Option<ToolFunction>,
}

impl ToolBuilder {
    /// Creates a new `ToolBuilder` with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool (e.g., "calculator", "weather_api")
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            parameters: HashMap::new(),
            function: None,
        }
    }

    /// Sets the description of the tool.
    ///
    /// # Arguments
    ///
    /// * `description` - A clear description of what the tool does
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Adds a parameter to the tool.
    ///
    /// # Arguments
    ///
    /// * `name` - The parameter name
    /// * `param_type` - The parameter type (e.g., "string", "number", "boolean", "object", "array")
    /// * `description` - A description of the parameter
    /// * `required` - Whether the parameter is required
    pub fn parameter(
        mut self,
        name: impl Into<String>,
        param_type: impl Into<String>,
        description: impl Into<String>,
        required: bool,
    ) -> Self {
        self.parameters.insert(
            name.into(),
            ToolParameter {
                param_type: param_type.into(),
                description: description.into(),
                required: Some(required),
            },
        );
        self
    }

    /// Adds an optional parameter to the tool (convenience method).
    ///
    /// # Arguments
    ///
    /// * `name` - The parameter name
    /// * `param_type` - The parameter type
    /// * `description` - A description of the parameter
    pub fn optional_parameter(
        self,
        name: impl Into<String>,
        param_type: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.parameter(name, param_type, description, false)
    }

    /// Adds a required parameter to the tool (convenience method).
    ///
    /// # Arguments
    ///
    /// * `name` - The parameter name
    /// * `param_type` - The parameter type
    /// * `description` - A description of the parameter
    pub fn required_parameter(
        self,
        name: impl Into<String>,
        param_type: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.parameter(name, param_type, description, true)
    }

    /// Sets the function to execute when the tool is called.
    ///
    /// The function should be an async function that takes `Value` (JSON arguments)
    /// and returns `Result<ToolResult>`.
    ///
    /// # Arguments
    ///
    /// * `f` - An async function that implements the tool's logic
    pub fn function<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<ToolResult>> + Send + 'static,
    {
        self.function = Some(Arc::new(move |args| Box::pin(f(args))));
        self
    }

    /// Sets the function using a synchronous closure.
    ///
    /// This is a convenience method for simple synchronous operations.
    ///
    /// # Arguments
    ///
    /// * `f` - A synchronous function that implements the tool's logic
    pub fn sync_function<F>(mut self, f: F) -> Self
    where
        F: Fn(Value) -> Result<ToolResult> + Send + Sync + 'static,
    {
        self.function = Some(Arc::new(move |args| {
            let result = f(args);
            Box::pin(async move { result })
        }));
        self
    }

    /// Builds the tool, consuming the builder and returning a boxed Tool.
    ///
    /// # Panics
    ///
    /// Panics if the function has not been set.
    pub fn build(self) -> Box<dyn Tool> {
        if self.function.is_none() {
            panic!("Tool function must be set before building");
        }

        Box::new(CustomTool {
            name: self.name,
            description: self.description,
            parameters: self.parameters,
            function: self.function.unwrap(),
        })
    }

    /// Builds the tool, returning a Result instead of panicking.
    ///
    /// Returns an error if the function has not been set.
    pub fn try_build(self) -> Result<Box<dyn Tool>> {
        if self.function.is_none() {
            return Err(HeliosError::ConfigError(
                "Tool function must be set before building".to_string(),
            ));
        }

        Ok(Box::new(CustomTool {
            name: self.name,
            description: self.description,
            parameters: self.parameters,
            function: self.function.unwrap(),
        }))
    }
}

/// Internal struct that wraps a custom tool created with ToolBuilder.
struct CustomTool {
    name: String,
    description: String,
    parameters: std::collections::HashMap<String, ToolParameter>,
    function: ToolFunction,
}

#[async_trait]
impl Tool for CustomTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        self.parameters.clone()
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        (self.function)(args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_basic_tool_builder() {
        async fn add_numbers(args: Value) -> Result<ToolResult> {
            let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
            Ok(ToolResult::success((a + b).to_string()))
        }

        let tool = ToolBuilder::new("add")
            .description("Add two numbers")
            .parameter("a", "number", "First number", true)
            .parameter("b", "number", "Second number", true)
            .function(add_numbers)
            .build();

        assert_eq!(tool.name(), "add");
        assert_eq!(tool.description(), "Add two numbers");

        let result = tool.execute(json!({ "a": 5.0, "b": 3.0 })).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "8");
    }

    #[tokio::test]
    async fn test_sync_function_builder() {
        let tool = ToolBuilder::new("echo")
            .description("Echo a message")
            .parameter("message", "string", "Message to echo", true)
            .sync_function(|args: Value| {
                let msg = args.get("message").and_then(|v| v.as_str()).unwrap_or("");
                Ok(ToolResult::success(format!("Echo: {}", msg)))
            })
            .build();

        let result = tool.execute(json!({ "message": "hello" })).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "Echo: hello");
    }

    #[tokio::test]
    async fn test_optional_parameters() {
        let tool = ToolBuilder::new("greet")
            .description("Greet someone")
            .required_parameter("name", "string", "Name of person to greet")
            .optional_parameter("title", "string", "Optional title (Mr, Mrs, etc)")
            .sync_function(|args: Value| {
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("stranger");
                let title = args.get("title").and_then(|v| v.as_str());

                let greeting = if let Some(t) = title {
                    format!("Hello, {} {}!", t, name)
                } else {
                    format!("Hello, {}!", name)
                };

                Ok(ToolResult::success(greeting))
            })
            .build();

        // Test with required parameter only
        let result1 = tool.execute(json!({ "name": "Alice" })).await.unwrap();
        assert_eq!(result1.output, "Hello, Alice!");

        // Test with both parameters
        let result2 = tool
            .execute(json!({ "name": "Smith", "title": "Dr" }))
            .await
            .unwrap();
        assert_eq!(result2.output, "Hello, Dr Smith!");
    }

    #[tokio::test]
    async fn test_closure_capture() {
        let multiplier = 10;

        let tool = ToolBuilder::new("multiply")
            .description("Multiply a number by a fixed value")
            .parameter("value", "number", "Value to multiply", true)
            .sync_function(move |args: Value| {
                let value = args.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                Ok(ToolResult::success((value * multiplier as f64).to_string()))
            })
            .build();

        let result = tool.execute(json!({ "value": 5.0 })).await.unwrap();
        assert_eq!(result.output, "50");
    }

    #[tokio::test]
    async fn test_error_handling() {
        let tool = ToolBuilder::new("fail")
            .description("A tool that fails")
            .sync_function(|_args: Value| {
                Err(HeliosError::ToolError("Intentional failure".to_string()))
            })
            .build();

        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
    }

    #[test]
    #[should_panic(expected = "Tool function must be set before building")]
    fn test_build_without_function() {
        let _tool = ToolBuilder::new("incomplete")
            .description("This will fail")
            .build();
    }

    #[tokio::test]
    async fn test_try_build_without_function() {
        let result = ToolBuilder::new("incomplete")
            .description("This will fail")
            .try_build();

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_json_arguments() {
        let tool = ToolBuilder::new("process_data")
            .description("Process complex JSON data")
            .parameter("data", "object", "Data object to process", true)
            .sync_function(|args: Value| {
                let data = args
                    .get("data")
                    .ok_or_else(|| HeliosError::ToolError("Missing data parameter".to_string()))?;

                let count = if let Some(obj) = data.as_object() {
                    obj.len()
                } else {
                    0
                };

                Ok(ToolResult::success(format!("Processed {} fields", count)))
            })
            .build();

        let result = tool
            .execute(json!({
                "data": {
                    "field1": "value1",
                    "field2": 42,
                    "field3": true
                }
            }))
            .await
            .unwrap();

        assert_eq!(result.output, "Processed 3 fields");
    }
}
