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
