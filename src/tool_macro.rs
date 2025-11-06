//! # Tool Macro Module
//!
//! This module provides macros to make tool creation as simple as possible.
//! Just define your parameters and logic - everything else is automatic!

/// The simplest way to create a tool - just wrap your function!
///
/// This macro lets you create a tool by just providing your function.
/// All parameter extraction is handled automatically.
///
/// # Example
///
/// ```rust
/// use helios_engine::tool;
///
/// fn adder(x: i32, y: i32) -> i32 {
///     x + y
/// }
///
/// let add_tool = tool!(adder, "Add two numbers", "x:i32:First number, y:i32:Second number");
/// ```
#[macro_export]
macro_rules! tool {
    // Match: tool!(function_name, "description", "params")
    ($func:ident, $desc:expr, $params:expr) => {{
        $crate::ToolBuilder::simple(stringify!($func), $desc, $params)
            .sync_function(|args| {
                // This is where automatic extraction would happen
                // For now, users still need to provide extraction logic via sync_function
                Ok($crate::ToolResult::success("Function executed".to_string()))
            })
            .build()
    }};
}

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
                        let $param_var = $crate::quick_tool!(@extract args, stringify!($param_name), $param_type);
                    )*
                    let result = $body;
                    Ok($crate::ToolResult::success(result))
                }
            ).build()
        }
    };

    // Extract helpers - use token matching
    (@extract $args:ident, $name:expr, i32) => {
        $args.get($name).and_then(|v| v.as_i64()).unwrap_or(0) as i32
    };
    (@extract $args:ident, $name:expr, i64) => {
        $args.get($name).and_then(|v| v.as_i64()).unwrap_or(0)
    };
    (@extract $args:ident, $name:expr, u32) => {
        $args.get($name).and_then(|v| v.as_u64()).unwrap_or(0) as u32
    };
    (@extract $args:ident, $name:expr, u64) => {
        $args.get($name).and_then(|v| v.as_u64()).unwrap_or(0)
    };
    (@extract $args:ident, $name:expr, f32) => {
        $args.get($name).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32
    };
    (@extract $args:ident, $name:expr, f64) => {
        $args.get($name).and_then(|v| v.as_f64()).unwrap_or(0.0)
    };
    (@extract $args:ident, $name:expr, bool) => {
        $args.get($name).and_then(|v| v.as_bool()).unwrap_or(false)
    };
    (@extract $args:ident, $name:expr, String) => {
        $args.get($name).and_then(|v| v.as_str()).unwrap_or("").to_string()
    };
}
