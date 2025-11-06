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
    parameter_order: Vec<String>,
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
            parameter_order: Vec::new(),
            function: None,
        }
    }

    /// Creates a `ToolBuilder` from an existing function, automatically deriving the tool name.
    ///
    /// This method extracts the function name and uses it as the tool name, making it
    /// extremely simple to convert existing functions into tools without redefining anything.
    ///
    /// # Arguments
    ///
    /// * `func_name` - The name to use for the tool (typically the function name)
    /// * `description` - A description of what the tool does
    /// * `params` - Parameter definitions in the format "name:type:description, ..."
    /// * `func` - The function to execute
    ///
    /// # Example
    ///
    /// ```rust
    /// use helios_engine::{ToolBuilder, ToolResult};
    ///
    /// fn calculate_area(length: f64, width: f64) -> f64 {
    ///     length * width
    /// }
    ///
    /// # async fn example() -> helios_engine::Result<()> {
    /// let tool = ToolBuilder::from_fn(
    ///     "calculate_area",
    ///     "Calculate the area of a rectangle",
    ///     "length:f64:The length of the rectangle, width:f64:The width of the rectangle",
    ///     |args| {
    ///         let length = args.get("length").and_then(|v| v.as_f64()).unwrap_or(0.0);
    ///         let width = args.get("width").and_then(|v| v.as_f64()).unwrap_or(0.0);
    ///         let area = calculate_area(length, width);
    ///         Ok(ToolResult::success(format!("The area is {} square units", area)))
    ///     }
    /// ).build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_fn<F>(
        func_name: impl Into<String>,
        description: impl Into<String>,
        params: impl Into<String>,
        func: F,
    ) -> Self
    where
        F: Fn(Value) -> Result<ToolResult> + Send + Sync + 'static,
    {
        Self::new(func_name)
            .description(description)
            .parameters(params)
            .sync_function(func)
    }

    /// Creates a tool from a simple function with automatic parameter extraction.
    ///
    /// This is the SIMPLEST API - just provide name, description, parameters, and the function.
    /// The function will be automatically wrapped and its parameters extracted from JSON.
    ///
    /// # Example with inline function
    ///
    /// ```rust
    /// use helios_engine::ToolBuilder;
    ///
    /// fn adder(x: i32, y: i32) -> i32 {
    ///     x + y
    /// }
    ///
    /// # async fn example() -> helios_engine::Result<()> {
    /// let tool = ToolBuilder::simple(
    ///     "add_numbers",
    ///     "Add two integers together",
    ///     "x:i32:First number, y:i32:Second number"
    /// ).build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn simple(
        name: impl Into<String>,
        description: impl Into<String>,
        params: impl Into<String>,
    ) -> Self {
        Self::new(name).description(description).parameters(params)
    }

    /// Creates a `ToolBuilder` from an existing async function, automatically deriving the tool name.
    ///
    /// This is the async version of `from_fn`, for functions that need to perform async operations.
    ///
    /// # Arguments
    ///
    /// * `func_name` - The name to use for the tool (typically the function name)
    /// * `description` - A description of what the tool does
    /// * `params` - Parameter definitions in the format "name:type:description, ..."
    /// * `func` - The async function to execute
    ///
    /// # Example
    ///
    /// ```rust
    /// use helios_engine::{ToolBuilder, ToolResult};
    ///
    /// async fn fetch_temperature(city: &str) -> Result<f64, String> {
    ///     // Simulate API call
    ///     Ok(20.5)
    /// }
    ///
    /// # async fn example() -> helios_engine::Result<()> {
    /// let tool = ToolBuilder::from_async_fn(
    ///     "fetch_temperature",
    ///     "Get the temperature for a city",
    ///     "city:string:The name of the city",
    ///     |args| async move {
    ///         let city = args.get("city").and_then(|v| v.as_str()).unwrap_or("");
    ///         let temp = fetch_temperature(city).await.unwrap_or(0.0);
    ///         Ok(ToolResult::success(format!("Temperature: {}°C", temp)))
    ///     }
    /// ).build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_async_fn<F, Fut>(
        func_name: impl Into<String>,
        description: impl Into<String>,
        params: impl Into<String>,
        func: F,
    ) -> Self
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<ToolResult>> + Send + 'static,
    {
        Self::new(func_name)
            .description(description)
            .parameters(params)
            .function(func)
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

    /// Adds multiple parameters at once using a compact format.
    ///
    /// This method allows you to define all parameters in a single string, making it much
    /// easier and more concise than calling `required_parameter` multiple times.
    ///
    /// # Format
    ///
    /// The format is: `"param_name:type:description, param_name2:type2:description2, ..."`
    ///
    /// Supported types:
    /// - `i32`, `i64`, `u32`, `u64`, `isize`, `usize` -> mapped to "integer"
    /// - `f32`, `f64`, `number` -> mapped to "number"
    /// - `str`, `String`, `string` -> mapped to "string"
    /// - `bool`, `boolean` -> mapped to "boolean"
    /// - `object` -> mapped to "object"
    /// - `array` -> mapped to "array"
    ///
    /// # Arguments
    ///
    /// * `params` - A comma-separated string of parameters in the format "name:type:description"
    ///
    /// # Example
    ///
    /// ```rust
    /// use helios_engine::ToolBuilder;
    ///
    /// let tool = ToolBuilder::new("calculate_volume")
    ///     .description("Calculate the volume of a box")
    ///     .parameters("width:i32:The width of the box, height:i32:The height of the box, depth:f64:The depth of the box")
    ///     .sync_function(|args| {
    ///         // function implementation
    /// #       Ok(helios_engine::ToolResult::success("done".to_string()))
    ///     })
    ///     .build();
    /// ```
    pub fn parameters(mut self, params: impl Into<String>) -> Self {
        let params_str = params.into();

        for param in params_str.split(',') {
            let param = param.trim();
            if param.is_empty() {
                continue;
            }

            let parts: Vec<&str> = param.splitn(3, ':').collect();
            if parts.len() < 2 {
                continue;
            }

            let name = parts[0].trim();
            let param_type = parts[1].trim();
            let description = if parts.len() >= 3 {
                parts[2].trim()
            } else {
                ""
            };

            // Map Rust types to JSON schema types
            let json_type = match param_type.to_lowercase().as_str() {
                "i32" | "i64" | "u32" | "u64" | "isize" | "usize" | "integer" => "integer",
                "f32" | "f64" | "number" => "number",
                "str" | "string" => "string",
                "bool" | "boolean" => "boolean",
                "object" => "object",
                "array" => "array",
                _ => param_type, // Use as-is if not recognized
            };

            let name_string = name.to_string();
            self.parameters.insert(
                name_string.clone(),
                ToolParameter {
                    param_type: json_type.to_string(),
                    description: description.to_string(),
                    required: Some(true),
                },
            );
            self.parameter_order.push(name_string);
        }

        self
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

    /// Ultra-simple API: Pass a function directly with automatic type inference.
    ///
    /// This method automatically infers parameter types from your function signature
    /// and extracts them from JSON. Works with any types that implement `FromValue`.
    ///
    /// Supported types: i32, i64, u32, u64, f32, f64, bool, String
    ///
    /// # Example
    ///
    /// ```rust
    /// use helios_engine::ToolBuilder;
    ///
    /// fn adder(x: i32, y: i32) -> i32 { x + y }
    /// fn greeter(name: String, formal: bool) -> String {
    ///     if formal {
    ///         format!("Good day, {}", name)
    ///     } else {
    ///         format!("Hey {}!", name)
    ///     }
    /// }
    ///
    /// # async fn example() -> helios_engine::Result<()> {
    /// let add_tool = ToolBuilder::new("add")
    ///     .description("Add two numbers")
    ///     .parameters("x:i32:First, y:i32:Second")
    ///     .ftool(adder)
    ///     .build();
    ///
    /// let greet_tool = ToolBuilder::new("greet")
    ///     .description("Greet someone")
    ///     .parameters("name:string:Name, formal:bool:Formal")
    ///     .ftool(greeter)
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn ftool<F, T1, T2, R>(self, f: F) -> Self
    where
        F: Fn(T1, T2) -> R + Send + Sync + 'static,
        T1: FromValue + Send + 'static,
        T2: FromValue + Send + 'static,
        R: ToString + Send + 'static,
    {
        let param_order = self.parameter_order.clone();
        self.sync_function(move |args| {
            let obj = args.as_object().ok_or_else(|| {
                HeliosError::ToolError("Expected JSON object for arguments".to_string())
            })?;

            if param_order.len() < 2 {
                return Ok(ToolResult::error("Expected at least 2 parameters"));
            }

            let p1 = obj
                .get(&param_order[0])
                .ok_or_else(|| {
                    HeliosError::ToolError(format!("Missing parameter: {}", param_order[0]))
                })?
                .clone();
            let p2 = obj
                .get(&param_order[1])
                .ok_or_else(|| {
                    HeliosError::ToolError(format!("Missing parameter: {}", param_order[1]))
                })?
                .clone();

            let p1 = T1::from_value(p1)?;
            let p2 = T2::from_value(p2)?;

            let result = f(p1, p2);
            Ok(ToolResult::success(result.to_string()))
        })
    }

    /// Ultra-simple API: Pass a 3-parameter function directly with automatic type inference.
    ///
    /// # Example
    ///
    /// ```rust
    /// use helios_engine::ToolBuilder;
    ///
    /// fn volume(width: f64, height: f64, depth: f64) -> f64 {
    ///     width * height * depth
    /// }
    ///
    /// # async fn example() -> helios_engine::Result<()> {
    /// let tool = ToolBuilder::new("calculate_volume")
    ///     .description("Calculate volume")
    ///     .parameters("width:f64:Width, height:f64:Height, depth:f64:Depth")
    ///     .ftool3(volume)
    ///     .build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn ftool3<F, T1, T2, T3, R>(self, f: F) -> Self
    where
        F: Fn(T1, T2, T3) -> R + Send + Sync + 'static,
        T1: FromValue + Send + 'static,
        T2: FromValue + Send + 'static,
        T3: FromValue + Send + 'static,
        R: ToString + Send + 'static,
    {
        let param_order = self.parameter_order.clone();
        self.sync_function(move |args| {
            let obj = args.as_object().ok_or_else(|| {
                HeliosError::ToolError("Expected JSON object for arguments".to_string())
            })?;

            if param_order.len() < 3 {
                return Ok(ToolResult::error("Expected at least 3 parameters"));
            }

            let p1 = obj
                .get(&param_order[0])
                .ok_or_else(|| {
                    HeliosError::ToolError(format!("Missing parameter: {}", param_order[0]))
                })?
                .clone();
            let p2 = obj
                .get(&param_order[1])
                .ok_or_else(|| {
                    HeliosError::ToolError(format!("Missing parameter: {}", param_order[1]))
                })?
                .clone();
            let p3 = obj
                .get(&param_order[2])
                .ok_or_else(|| {
                    HeliosError::ToolError(format!("Missing parameter: {}", param_order[2]))
                })?
                .clone();

            let p1 = T1::from_value(p1)?;
            let p2 = T2::from_value(p2)?;
            let p3 = T3::from_value(p3)?;

            let result = f(p1, p2, p3);
            Ok(ToolResult::success(result.to_string()))
        })
    }

    /// Ultra-simple API: Pass a 4-parameter function directly with automatic type inference.
    pub fn ftool4<F, T1, T2, T3, T4, R>(self, f: F) -> Self
    where
        F: Fn(T1, T2, T3, T4) -> R + Send + Sync + 'static,
        T1: FromValue + Send + 'static,
        T2: FromValue + Send + 'static,
        T3: FromValue + Send + 'static,
        T4: FromValue + Send + 'static,
        R: ToString + Send + 'static,
    {
        let param_order = self.parameter_order.clone();
        self.sync_function(move |args| {
            let obj = args.as_object().ok_or_else(|| {
                HeliosError::ToolError("Expected JSON object for arguments".to_string())
            })?;

            if param_order.len() < 4 {
                return Ok(ToolResult::error("Expected at least 4 parameters"));
            }

            let p1 = T1::from_value(obj.get(&param_order[0]).cloned().unwrap_or(Value::Null))?;
            let p2 = T2::from_value(obj.get(&param_order[1]).cloned().unwrap_or(Value::Null))?;
            let p3 = T3::from_value(obj.get(&param_order[2]).cloned().unwrap_or(Value::Null))?;
            let p4 = T4::from_value(obj.get(&param_order[3]).cloned().unwrap_or(Value::Null))?;

            let result = f(p1, p2, p3, p4);
            Ok(ToolResult::success(result.to_string()))
        })
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

/// Trait for converting JSON values to Rust types.
/// This enables automatic type inference in the ftool API.
pub trait FromValue: Sized {
    fn from_value(value: Value) -> Result<Self>;
}

impl FromValue for i32 {
    fn from_value(value: Value) -> Result<Self> {
        value
            .as_i64()
            .map(|n| n as i32)
            .ok_or_else(|| HeliosError::ToolError("Expected integer value".to_string()))
    }
}

impl FromValue for i64 {
    fn from_value(value: Value) -> Result<Self> {
        value
            .as_i64()
            .ok_or_else(|| HeliosError::ToolError("Expected integer value".to_string()))
    }
}

impl FromValue for u32 {
    fn from_value(value: Value) -> Result<Self> {
        value
            .as_u64()
            .map(|n| n as u32)
            .ok_or_else(|| HeliosError::ToolError("Expected unsigned integer value".to_string()))
    }
}

impl FromValue for u64 {
    fn from_value(value: Value) -> Result<Self> {
        value
            .as_u64()
            .ok_or_else(|| HeliosError::ToolError("Expected unsigned integer value".to_string()))
    }
}

impl FromValue for f32 {
    fn from_value(value: Value) -> Result<Self> {
        value
            .as_f64()
            .map(|n| n as f32)
            .ok_or_else(|| HeliosError::ToolError("Expected float value".to_string()))
    }
}

impl FromValue for f64 {
    fn from_value(value: Value) -> Result<Self> {
        value
            .as_f64()
            .ok_or_else(|| HeliosError::ToolError("Expected float value".to_string()))
    }
}

impl FromValue for bool {
    fn from_value(value: Value) -> Result<Self> {
        value
            .as_bool()
            .ok_or_else(|| HeliosError::ToolError("Expected boolean value".to_string()))
    }
}

impl FromValue for String {
    fn from_value(value: Value) -> Result<Self> {
        value
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| HeliosError::ToolError("Expected string value".to_string()))
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

    #[tokio::test]
    async fn test_parameters_method() {
        let tool = ToolBuilder::new("calculate_area")
            .description("Calculate area of a rectangle")
            .parameters("width:i32:The width, height:i32:The height")
            .sync_function(|args: Value| {
                let width = args.get("width").and_then(|v| v.as_i64()).unwrap_or(0);
                let height = args.get("height").and_then(|v| v.as_i64()).unwrap_or(0);
                Ok(ToolResult::success(format!("Area: {}", width * height)))
            })
            .build();

        assert_eq!(tool.name(), "calculate_area");

        let params = tool.parameters();
        assert!(params.contains_key("width"));
        assert!(params.contains_key("height"));
        assert_eq!(params.get("width").unwrap().param_type, "integer");
        assert_eq!(params.get("height").unwrap().param_type, "integer");

        let result = tool
            .execute(json!({"width": 5, "height": 10}))
            .await
            .unwrap();
        assert_eq!(result.output, "Area: 50");
    }

    #[tokio::test]
    async fn test_parameters_with_float_types() {
        let tool = ToolBuilder::new("calculate_volume")
            .description("Calculate volume")
            .parameters("width:f64:Width in meters, height:f32:Height in meters, depth:number:Depth in meters")
            .sync_function(|args: Value| {
                let width = args.get("width").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let height = args.get("height").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let depth = args.get("depth").and_then(|v| v.as_f64()).unwrap_or(0.0);
                Ok(ToolResult::success(format!("Volume: {:.2}", width * height * depth)))
            })
            .build();

        let params = tool.parameters();
        assert_eq!(params.get("width").unwrap().param_type, "number");
        assert_eq!(params.get("height").unwrap().param_type, "number");
        assert_eq!(params.get("depth").unwrap().param_type, "number");
    }

    #[tokio::test]
    async fn test_parameters_with_string_and_bool() {
        let tool = ToolBuilder::new("greet")
            .description("Greet someone")
            .parameters("name:string:Person's name, formal:bool:Use formal greeting")
            .sync_function(|args: Value| {
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("stranger");
                let formal = args
                    .get("formal")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let greeting = if formal {
                    format!("Good day, {}", name)
                } else {
                    format!("Hey {}", name)
                };
                Ok(ToolResult::success(greeting))
            })
            .build();

        let params = tool.parameters();
        assert_eq!(params.get("name").unwrap().param_type, "string");
        assert_eq!(params.get("formal").unwrap().param_type, "boolean");
    }

    #[tokio::test]
    async fn test_from_fn() {
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }

        let tool = ToolBuilder::from_fn(
            "add",
            "Add two numbers",
            "a:i32:First number, b:i32:Second number",
            |args| {
                let a = args.get("a").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let b = args.get("b").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                Ok(ToolResult::success(add(a, b).to_string()))
            },
        )
        .build();

        assert_eq!(tool.name(), "add");
        assert_eq!(tool.description(), "Add two numbers");

        let result = tool.execute(json!({"a": 3, "b": 7})).await.unwrap();
        assert_eq!(result.output, "10");
    }

    #[tokio::test]
    async fn test_from_async_fn() {
        async fn fetch_data(id: i32) -> String {
            format!("Data for ID: {}", id)
        }

        let tool = ToolBuilder::from_async_fn(
            "fetch_data",
            "Fetch data by ID",
            "id:i32:The ID to fetch",
            |args| async move {
                let id = args.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                Ok(ToolResult::success(fetch_data(id).await))
            },
        )
        .build();

        assert_eq!(tool.name(), "fetch_data");

        let result = tool.execute(json!({"id": 42})).await.unwrap();
        assert_eq!(result.output, "Data for ID: 42");
    }

    #[tokio::test]
    async fn test_parameters_empty_and_whitespace() {
        let tool = ToolBuilder::new("test")
            .description("Test tool")
            .parameters("a:i32:First, , b:i32:Second,  ,  c:string:Third  ")
            .sync_function(|_| Ok(ToolResult::success("ok".to_string())))
            .build();

        let params = tool.parameters();
        // Should have 3 parameters (empty strings should be skipped)
        assert_eq!(params.len(), 3);
        assert!(params.contains_key("a"));
        assert!(params.contains_key("b"));
        assert!(params.contains_key("c"));
    }

    #[tokio::test]
    async fn test_parameters_without_description() {
        let tool = ToolBuilder::new("test")
            .description("Test tool")
            .parameters("x:i32, y:i32")
            .sync_function(|_| Ok(ToolResult::success("ok".to_string())))
            .build();

        let params = tool.parameters();
        assert_eq!(params.len(), 2);
        assert_eq!(params.get("x").unwrap().description, "");
        assert_eq!(params.get("y").unwrap().description, "");
    }
}
