//! # Tool Macro Module
//!
//! This module provides macros to make tool creation as simple as possible.
//! Just define your parameters and logic - everything else is automatic!

/// Quick tool creation with auto-derived types.
///
/// This is the simplest way to create a tool - just provide the function signature
/// and body, and everything else is handled automatically.
///
/// # Example
///
/// ```rust
/// use helios_engine::quick_tool;
///
/// let tool = quick_tool! {
///     name: calculate_volume,
///     description: "Calculate the volume of a box",
///     params: (width: f64, height: f64, depth: f64),
///     execute: |width, height, depth| {
///         format!("Volume: {} cubic meters", width * height * depth)
///     }
/// };
/// ```
#[macro_export]
macro_rules! quick_tool {
    (
        name: $name:ident,
        description: $desc:expr,
        params: ($($param_name:ident: $param_type:tt),* $(,)?),
        execute: |$($param_var:ident),*| $body:expr
    ) => {
        {
            let param_str = vec![
                $(
                    format!(
                        "{}:{}:{}",
                        stringify!($param_name),
                        stringify!($param_type),
                        stringify!($param_name).replace('_', " ")
                    )
                ),*
            ].join(", ");

            $crate::ToolBuilder::from_fn(
                stringify!($name),
                $desc,
                param_str,
                |args| {
                    $(
                        let $param_var = $crate::quick_tool!(@extract args, stringify!($param_name), $param_type)?;
                    )*
                    let result = $body;
                    Ok($crate::ToolResult::success(result))
                }
            ).build()
        }
    };

    // Extract helpers - return errors for missing required parameters
    (@extract $args:ident, $name:expr, i32) => {
        $args.get($name)
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .ok_or_else(|| $crate::error::HeliosError::ToolError(
                format!("Missing or invalid required parameter '{}'", $name)
            ))
    };
    (@extract $args:ident, $name:expr, i64) => {
        $args.get($name)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| $crate::error::HeliosError::ToolError(
                format!("Missing or invalid required parameter '{}'", $name)
            ))
    };
    (@extract $args:ident, $name:expr, u32) => {
        $args.get($name)
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .ok_or_else(|| $crate::error::HeliosError::ToolError(
                format!("Missing or invalid required parameter '{}'", $name)
            ))
    };
    (@extract $args:ident, $name:expr, u64) => {
        $args.get($name)
            .and_then(|v| v.as_u64())
            .ok_or_else(|| $crate::error::HeliosError::ToolError(
                format!("Missing or invalid required parameter '{}'", $name)
            ))
    };
    (@extract $args:ident, $name:expr, f32) => {
        $args.get($name)
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .ok_or_else(|| $crate::error::HeliosError::ToolError(
                format!("Missing or invalid required parameter '{}'", $name)
            ))
    };
    (@extract $args:ident, $name:expr, f64) => {
        $args.get($name)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| $crate::error::HeliosError::ToolError(
                format!("Missing or invalid required parameter '{}'", $name)
            ))
    };
    (@extract $args:ident, $name:expr, bool) => {
        $args.get($name)
            .and_then(|v| v.as_bool())
            .ok_or_else(|| $crate::error::HeliosError::ToolError(
                format!("Missing or invalid required parameter '{}'", $name)
            ))
    };
    (@extract $args:ident, $name:expr, String) => {
        $args.get($name)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| $crate::error::HeliosError::ToolError(
                format!("Missing or invalid required parameter '{}'", $name)
            ))
    };
}

#[cfg(test)]
mod tests {
    use crate::quick_tool;
    use serde_json::json;

    #[test]
    fn test_quick_tool_with_valid_parameters() {
        let tool = quick_tool! {
            name: test_add,
            description: "Add two numbers",
            params: (x: i32, y: i32),
            execute: |x, y| {
                format!("Result: {}", x + y)
            }
        };

        assert_eq!(tool.name(), "test_add");
        assert_eq!(tool.description(), "Add two numbers");
    }

    #[tokio::test]
    async fn test_quick_tool_execution_with_valid_args() {
        let tool = quick_tool! {
            name: test_multiply,
            description: "Multiply two numbers",
            params: (a: i32, b: i32),
            execute: |a, b| {
                format!("{}", a * b)
            }
        };

        let args = json!({"a": 5, "b": 3});
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "15");
    }

    #[tokio::test]
    async fn test_quick_tool_missing_parameter_returns_error() {
        let tool = quick_tool! {
            name: test_divide,
            description: "Divide two numbers",
            params: (numerator: f64, denominator: f64),
            execute: |num, den| {
                format!("{}", num / den)
            }
        };

        // Missing 'denominator' parameter
        let args = json!({"numerator": 10.0});
        let result = tool.execute(args).await;

        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Missing or invalid required parameter"));
    }

    #[tokio::test]
    async fn test_quick_tool_invalid_type_returns_error() {
        let tool = quick_tool! {
            name: test_type_check,
            description: "Test type checking",
            params: (value: i32),
            execute: |v| {
                format!("{}", v)
            }
        };

        // Passing string instead of integer
        let args = json!({"value": "not a number"});
        let result = tool.execute(args).await;

        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Missing or invalid required parameter"));
    }

    #[tokio::test]
    async fn test_quick_tool_with_string_and_bool() {
        let tool = quick_tool! {
            name: test_greet,
            description: "Greet someone",
            params: (name: String, formal: bool),
            execute: |name, formal| {
                if formal {
                    format!("Good day, {}", name)
                } else {
                    format!("Hey {}", name)
                }
            }
        };

        let args = json!({"name": "Alice", "formal": true});
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "Good day, Alice");

        let args = json!({"name": "Bob", "formal": false});
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "Hey Bob");
    }

    #[tokio::test]
    async fn test_quick_tool_prevents_division_by_zero_default() {
        // This test verifies that missing parameters cause errors
        // instead of defaulting to 0, which would cause division by zero
        let tool = quick_tool! {
            name: safe_divide,
            description: "Safely divide two numbers",
            params: (x: f64, y: f64),
            execute: |x, y| {
                format!("{}", x / y)
            }
        };

        // Missing divisor should return error, not default to 0.0
        let args = json!({"x": 10.0});
        let result = tool.execute(args).await;

        assert!(
            result.is_err(),
            "Should error on missing divisor, not default to 0"
        );
    }
}
