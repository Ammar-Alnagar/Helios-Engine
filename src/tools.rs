//! # Tools Module
//!
//! This module provides the framework for creating and managing tools that can be used by agents.
//! It defines the `Tool` trait, which all tools must implement, and the `ToolRegistry`
//! for managing a collection of tools.
//! It also includes several built-in tools for common tasks.

use crate::error::{HeliosError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// A parameter for a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    /// The type of the parameter (e.g., "string", "number").
    #[serde(rename = "type")]
    pub param_type: String,
    /// A description of the parameter.
    pub description: String,
    /// Whether the parameter is required.
    #[serde(skip)]
    pub required: Option<bool>,
}

/// The definition of a tool that can be sent to an LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// The type of the tool (e.g., "function").
    #[serde(rename = "type")]
    pub tool_type: String,
    /// The function definition for the tool.
    pub function: FunctionDefinition,
}

/// The definition of a function that can be called by a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// The name of the function.
    pub name: String,
    /// A description of the function.
    pub description: String,
    /// The parameters for the function.
    pub parameters: ParametersSchema,
}

/// The schema for the parameters of a function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParametersSchema {
    /// The type of the schema (should be "object").
    #[serde(rename = "type")]
    pub schema_type: String,
    /// The properties of the schema.
    pub properties: HashMap<String, ToolParameter>,
    /// The required properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

/// The result of a tool execution.
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// Whether the execution was successful.
    pub success: bool,
    /// The output of the execution.
    pub output: String,
}

impl ToolResult {
    /// Creates a new successful `ToolResult`.
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
        }
    }

    /// Creates a new error `ToolResult`.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            output: message.into(),
        }
    }
}

/// A trait for tools that can be used by agents.
#[async_trait]
pub trait Tool: Send + Sync {
    /// The name of the tool.
    fn name(&self) -> &str;
    /// A description of the tool.
    fn description(&self) -> &str;
    /// The parameters for the tool.
    fn parameters(&self) -> HashMap<String, ToolParameter>;
    /// Executes the tool with the given arguments.
    async fn execute(&self, args: Value) -> Result<ToolResult>;

    /// Converts the tool to a `ToolDefinition`.
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
                    required: if required.is_empty() {
                        None
                    } else {
                        Some(required)
                    },
                },
            },
        }
    }
}

/// A registry for managing a collection of tools.
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Creates a new `ToolRegistry`.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Registers a tool with the registry.
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    /// Gets a tool from the registry by name.
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|b| &**b)
    }

    /// Executes a tool in the registry by name.
    pub async fn execute(&self, name: &str, args: Value) -> Result<ToolResult> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| HeliosError::ToolError(format!("Tool '{}' not found", name)))?;

        tool.execute(args).await
    }

    /// Gets the definitions of all tools in the registry.
    pub fn get_definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|tool| tool.to_definition())
            .collect()
    }

    /// Lists the names of all tools in the registry.
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

/// A tool for performing basic arithmetic operations.
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

/// Evaluates a simple mathematical expression.
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

/// A tool that echoes back the provided message.
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

/// A tool for searching for files.
pub struct FileSearchTool;

#[async_trait]
impl Tool for FileSearchTool {
    fn name(&self) -> &str {
        "file_search"
    }

    fn description(&self) -> &str {
        "Search for files by name pattern or search for content within files. Can search recursively in directories."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "path".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "The directory path to search in (default: current directory)"
                    .to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "pattern".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "File name pattern to search for (supports wildcards like *.rs)"
                    .to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "content".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Text content to search for within files".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "max_results".to_string(),
            ToolParameter {
                param_type: "number".to_string(),
                description: "Maximum number of results to return (default: 50)".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        use walkdir::WalkDir;

        let base_path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");

        let pattern = args.get("pattern").and_then(|v| v.as_str());
        let content_search = args.get("content").and_then(|v| v.as_str());
        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as usize;

        if pattern.is_none() && content_search.is_none() {
            return Err(HeliosError::ToolError(
                "Either 'pattern' or 'content' parameter is required".to_string(),
            ));
        }

        let mut results = Vec::new();

        // Precompile filename pattern to avoid compiling per file
        let compiled_re = if let Some(pat) = pattern {
            let re_pattern = pat.replace(".", r"\.").replace("*", ".*").replace("?", ".");
            match regex::Regex::new(&format!("^{}$", re_pattern)) {
                Ok(re) => Some(re),
                Err(e) => {
                    tracing::warn!(
                        "Invalid glob pattern '{}' ({}). Falling back to substring matching.",
                        pat,
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

        for entry in WalkDir::new(base_path)
            .max_depth(10)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if results.len() >= max_results {
                break;
            }

            let path = entry.path();

            // Skip hidden files and common ignore directories
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with('.')
                    || file_name == "target"
                    || file_name == "node_modules"
                    || file_name == "__pycache__"
                {
                    continue;
                }
            }

            // Pattern matching for file names
            if let Some(pat) = pattern {
                if path.is_file() {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        let is_match = if let Some(re) = &compiled_re {
                            re.is_match(file_name)
                        } else {
                            file_name.contains(pat)
                        };
                        if is_match {
                            results.push(format!("📄 {}", path.display()));
                        }
                    }
                }
            }

            // Content search within files
            if let Some(search_term) = content_search {
                if path.is_file() {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        if content.contains(search_term) {
                            // Find line numbers where content appears
                            let matching_lines: Vec<(usize, &str)> = content
                                .lines()
                                .enumerate()
                                .filter(|(_, line)| line.contains(search_term))
                                .take(3) // Show up to 3 matching lines per file
                                .collect();

                            if !matching_lines.is_empty() {
                                results.push(format!(
                                    "📄 {} (found in {} lines)",
                                    path.display(),
                                    matching_lines.len()
                                ));
                                for (line_num, line) in matching_lines {
                                    results.push(format!(
                                        "  Line {}: {}",
                                        line_num + 1,
                                        line.trim()
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        if results.is_empty() {
            Ok(ToolResult::success(
                "No files found matching the criteria.".to_string(),
            ))
        } else {
            let output = format!(
                "Found {} result(s):\n\n{}",
                results.len(),
                results.join("\n")
            );
            Ok(ToolResult::success(output))
        }
    }
}

// (removed) glob_match helper – logic moved to precompiled regex in FileSearchTool::execute

/// A tool for reading the contents of a file.
pub struct FileReadTool;

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read the contents of a file. Returns the full file content or specific lines."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "path".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "The file path to read".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "start_line".to_string(),
            ToolParameter {
                param_type: "number".to_string(),
                description: "Starting line number (1-indexed, optional)".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "end_line".to_string(),
            ToolParameter {
                param_type: "number".to_string(),
                description: "Ending line number (1-indexed, optional)".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let file_path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'path' parameter".to_string()))?;

        let content = std::fs::read_to_string(file_path)
            .map_err(|e| HeliosError::ToolError(format!("Failed to read file: {}", e)))?;

        let start_line = args
            .get("start_line")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize);
        let end_line = args
            .get("end_line")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize);

        let output = if let (Some(start), Some(end)) = (start_line, end_line) {
            let lines: Vec<&str> = content.lines().collect();
            let start_idx = start.saturating_sub(1);
            let end_idx = end.min(lines.len());

            if start_idx >= lines.len() {
                return Err(HeliosError::ToolError(format!(
                    "Start line {} is beyond file length ({})",
                    start,
                    lines.len()
                )));
            }

            let selected_lines = &lines[start_idx..end_idx];
            format!(
                "File: {} (lines {}-{}):\n\n{}",
                file_path,
                start,
                end_idx,
                selected_lines.join("\n")
            )
        } else {
            format!("File: {}:\n\n{}", file_path, content)
        };

        Ok(ToolResult::success(output))
    }
}

/// A tool for writing content to a file.
pub struct FileWriteTool;

#[async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Write content to a file. Creates new file or overwrites existing file."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "path".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "The file path to write to".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "content".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "The content to write to the file".to_string(),
                required: Some(true),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let file_path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'path' parameter".to_string()))?;

        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'content' parameter".to_string()))?;

        // Create parent directories if they don't exist
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                HeliosError::ToolError(format!("Failed to create directories: {}", e))
            })?;
        }

        std::fs::write(file_path, content)
            .map_err(|e| HeliosError::ToolError(format!("Failed to write file: {}", e)))?;

        Ok(ToolResult::success(format!(
            "Successfully wrote {} bytes to {}",
            content.len(),
            file_path
        )))
    }
}

/// A tool for editing a file by replacing text.
pub struct FileEditTool;

#[async_trait]
impl Tool for FileEditTool {
    fn name(&self) -> &str {
        "file_edit"
    }

    fn description(&self) -> &str {
        "Edit a file by replacing specific text or lines. Use this to make targeted changes to existing files."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "path".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "The file path to edit".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "find".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "The text to find and replace".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "replace".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "The replacement text".to_string(),
                required: Some(true),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let file_path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'path' parameter".to_string()))?;

        let find_text = args
            .get("find")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'find' parameter".to_string()))?;

        let replace_text = args
            .get("replace")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'replace' parameter".to_string()))?;

        if find_text.is_empty() {
            return Err(HeliosError::ToolError(
                "'find' parameter cannot be empty".to_string(),
            ));
        }

        let path = Path::new(file_path);
        let parent = path
            .parent()
            .ok_or_else(|| HeliosError::ToolError(format!("Invalid target path: {}", file_path)))?;
        let file_name = path
            .file_name()
            .ok_or_else(|| HeliosError::ToolError(format!("Invalid target path: {}", file_path)))?;

        // Build a temp file path in the same directory for atomic rename
        let pid = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| HeliosError::ToolError(format!("Clock error: {}", e)))?
            .as_nanos();
        let tmp_name = format!("{}.tmp.{}.{}", file_name.to_string_lossy(), pid, nanos);
        let tmp_path = parent.join(tmp_name);

        // Open files
        let input_file = std::fs::File::open(path)
            .map_err(|e| HeliosError::ToolError(format!("Failed to open file for read: {}", e)))?;
        let mut reader = BufReader::new(input_file);

        let tmp_file = std::fs::File::create(&tmp_path).map_err(|e| {
            HeliosError::ToolError(format!(
                "Failed to create temp file {}: {}",
                tmp_path.display(),
                e
            ))
        })?;
        let mut writer = BufWriter::new(&tmp_file);

        // Streamed find/replace to avoid loading entire file into memory
        let replaced_count = replace_streaming(
            &mut reader,
            &mut writer,
            find_text.as_bytes(),
            replace_text.as_bytes(),
        )
        .map_err(|e| HeliosError::ToolError(format!("I/O error while replacing: {}", e)))?;

        // Ensure all data is flushed and synced before rename
        writer
            .flush()
            .map_err(|e| HeliosError::ToolError(format!("Failed to flush temp file: {}", e)))?;
        tmp_file
            .sync_all()
            .map_err(|e| HeliosError::ToolError(format!("Failed to sync temp file: {}", e)))?;

        // Preserve permissions
        if let Ok(meta) = std::fs::metadata(path) {
            if let Err(e) = std::fs::set_permissions(&tmp_path, meta.permissions()) {
                let _ = std::fs::remove_file(&tmp_path);
                return Err(HeliosError::ToolError(format!(
                    "Failed to set permissions: {}",
                    e
                )));
            }
        }

        // Atomic replace
        std::fs::rename(&tmp_path, path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp_path);
            HeliosError::ToolError(format!("Failed to replace original file: {}", e))
        })?;

        if replaced_count == 0 {
            return Ok(ToolResult::error(format!(
                "Text '{}' not found in file {}",
                find_text, file_path
            )));
        }

        Ok(ToolResult::success(format!(
            "Successfully replaced {} occurrence(s) in {}",
            replaced_count, file_path
        )))
    }
}

/// Performs a streaming replacement of a needle in a reader, writing to a writer.
fn replace_streaming<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    needle: &[u8],
    replacement: &[u8],
) -> std::io::Result<usize> {
    let mut replaced = 0usize;
    let mut carry: Vec<u8> = Vec::new();
    let mut buf = [0u8; 8192];

    let tail = if needle.len() > 1 {
        needle.len() - 1
    } else {
        0
    };

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }

        let mut combined = Vec::with_capacity(carry.len() + n);
        combined.extend_from_slice(&carry);
        combined.extend_from_slice(&buf[..n]);

        let process_len = combined.len().saturating_sub(tail);
        let (to_process, new_carry) = combined.split_at(process_len);
        replaced += write_with_replacements(writer, to_process, needle, replacement)?;
        carry.clear();
        carry.extend_from_slice(new_carry);
    }

    // Process remaining carry fully
    replaced += write_with_replacements(writer, &carry, needle, replacement)?;
    Ok(replaced)
}

/// Writes the haystack to the writer, replacing all occurrences of the needle with the replacement.
fn write_with_replacements<W: Write>(
    writer: &mut W,
    haystack: &[u8],
    needle: &[u8],
    replacement: &[u8],
) -> std::io::Result<usize> {
    if needle.is_empty() {
        writer.write_all(haystack)?;
        return Ok(0);
    }

    let mut count = 0usize;
    let mut i = 0usize;
    while let Some(pos) = find_subslice(&haystack[i..], needle) {
        let idx = i + pos;
        writer.write_all(&haystack[i..idx])?;
        writer.write_all(replacement)?;
        count += 1;
        i = idx + needle.len();
    }
    writer.write_all(&haystack[i..])?;
    Ok(count)
}

/// Finds the first occurrence of a subslice in a slice.
fn find_subslice(h: &[u8], n: &[u8]) -> Option<usize> {
    if n.is_empty() {
        return Some(0);
    }
    h.windows(n.len()).position(|w| w == n)
}

/// RAG (Retrieval-Augmented Generation) Tool with Qdrant Vector Database
///
/// Provides document embedding, storage, retrieval, and reranking capabilities.
/// Supports operations: add_document, search, delete, list, clear
#[derive(Clone)]
pub struct QdrantRAGTool {
    qdrant_url: String,
    collection_name: String,
    embedding_api_url: String,
    embedding_api_key: String,
    client: reqwest::Client,
}

/// A point in a Qdrant collection.
#[derive(Debug, Serialize, Deserialize)]
struct QdrantPoint {
    id: String,
    vector: Vec<f32>,
    payload: HashMap<String, serde_json::Value>,
}

/// A search request to a Qdrant collection.
#[derive(Debug, Serialize, Deserialize)]
struct QdrantSearchRequest {
    vector: Vec<f32>,
    limit: usize,
    with_payload: bool,
    with_vector: bool,
}

/// A search response from a Qdrant collection.
#[derive(Debug, Serialize, Deserialize)]
struct QdrantSearchResponse {
    result: Vec<QdrantSearchResult>,
}

/// A search result from a Qdrant collection.
#[derive(Debug, Serialize, Deserialize)]
struct QdrantSearchResult {
    id: String,
    score: f64,
    payload: Option<HashMap<String, serde_json::Value>>,
}

/// A request to an embedding API.
#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
}

/// A response from an embedding API.
#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

/// The data for an embedding.
#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

impl QdrantRAGTool {
    /// Creates a new `QdrantRAGTool`.
    pub fn new(
        qdrant_url: impl Into<String>,
        collection_name: impl Into<String>,
        embedding_api_url: impl Into<String>,
        embedding_api_key: impl Into<String>,
    ) -> Self {
        Self {
            qdrant_url: qdrant_url.into(),
            collection_name: collection_name.into(),
            embedding_api_url: embedding_api_url.into(),
            embedding_api_key: embedding_api_key.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Generates an embedding for the given text.
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = EmbeddingRequest {
            input: text.to_string(),
            model: "text-embedding-ada-002".to_string(),
        };

        let response = self
            .client
            .post(&self.embedding_api_url)
            .header(
                "Authorization",
                format!("Bearer {}", self.embedding_api_key),
            )
            .json(&request)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Embedding API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!(
                "Embedding failed: {}",
                error_text
            )));
        }

        let embedding_response: EmbeddingResponse = response.json().await.map_err(|e| {
            HeliosError::ToolError(format!("Failed to parse embedding response: {}", e))
        })?;

        embedding_response
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| HeliosError::ToolError("No embedding returned".to_string()))
    }

    /// Ensures that the Qdrant collection exists.
    async fn ensure_collection(&self) -> Result<()> {
        let collection_url = format!("{}/collections/{}", self.qdrant_url, self.collection_name);

        // Check if collection exists
        let response = self.client.get(&collection_url).send().await;

        if response.is_ok() && response.unwrap().status().is_success() {
            return Ok(()); // Collection exists
        }

        // Create collection with 1536 dimensions (OpenAI embedding size)
        let create_payload = serde_json::json!({
            "vectors": {
                "size": 1536,
                "distance": "Cosine"
            }
        });

        let response = self
            .client
            .put(&collection_url)
            .json(&create_payload)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Failed to create collection: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!(
                "Collection creation failed: {}",
                error_text
            )));
        }

        Ok(())
    }

    /// Adds a document to the Qdrant collection.
    async fn add_document(
        &self,
        text: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        self.ensure_collection().await?;

        // Generate embedding
        let embedding = self.generate_embedding(text).await?;

        // Create point with metadata
        let point_id = Uuid::new_v4().to_string();
        let mut payload = metadata;
        payload.insert("text".to_string(), serde_json::json!(text));
        payload.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now().to_rfc3339()),
        );

        let point = QdrantPoint {
            id: point_id.clone(),
            vector: embedding,
            payload,
        };

        // Upload point to Qdrant
        let upsert_url = format!(
            "{}/collections/{}/points",
            self.qdrant_url, self.collection_name
        );
        let upsert_payload = serde_json::json!({
            "points": [point]
        });

        let response = self
            .client
            .put(&upsert_url)
            .json(&upsert_payload)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Failed to upload document: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!(
                "Document upload failed: {}",
                error_text
            )));
        }

        Ok(point_id)
    }

    /// Searches for similar documents in the Qdrant collection.
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<(String, f64, String)>> {
        // Generate query embedding
        let query_embedding = self.generate_embedding(query).await?;

        // Search in Qdrant
        let search_url = format!(
            "{}/collections/{}/points/search",
            self.qdrant_url, self.collection_name
        );
        let search_request = QdrantSearchRequest {
            vector: query_embedding,
            limit,
            with_payload: true,
            with_vector: false,
        };

        let response = self
            .client
            .post(&search_url)
            .json(&search_request)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Search failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!(
                "Search request failed: {}",
                error_text
            )));
        }

        let search_response: QdrantSearchResponse = response.json().await.map_err(|e| {
            HeliosError::ToolError(format!("Failed to parse search response: {}", e))
        })?;

        // Extract results
        let results: Vec<(String, f64, String)> = search_response
            .result
            .into_iter()
            .filter_map(|r| {
                r.payload.and_then(|p| {
                    p.get("text")
                        .and_then(|t| t.as_str())
                        .map(|text| (r.id, r.score, text.to_string()))
                })
            })
            .collect();

        Ok(results)
    }

    /// Deletes a document from the Qdrant collection by ID.
    async fn delete_document(&self, doc_id: &str) -> Result<()> {
        let delete_url = format!(
            "{}/collections/{}/points/delete",
            self.qdrant_url, self.collection_name
        );
        let delete_payload = serde_json::json!({
            "points": [doc_id]
        });

        let response = self
            .client
            .post(&delete_url)
            .json(&delete_payload)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Delete failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!(
                "Delete request failed: {}",
                error_text
            )));
        }

        Ok(())
    }

    /// Clears all documents from the Qdrant collection.
    async fn clear_collection(&self) -> Result<()> {
        let delete_url = format!("{}/collections/{}", self.qdrant_url, self.collection_name);

        let response = self
            .client
            .delete(&delete_url)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Clear failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!(
                "Clear collection failed: {}",
                error_text
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl Tool for QdrantRAGTool {
    fn name(&self) -> &str {
        "rag_qdrant"
    }

    fn description(&self) -> &str {
        "RAG (Retrieval-Augmented Generation) tool with vector database. Operations: add_document, search, delete, clear"
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "operation".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Operation: 'add_document', 'search', 'delete', 'clear'".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "text".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Text content for add_document or search query".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "doc_id".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Document ID for delete operation".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "limit".to_string(),
            ToolParameter {
                param_type: "number".to_string(),
                description: "Number of results for search (default: 5)".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "metadata".to_string(),
            ToolParameter {
                param_type: "object".to_string(),
                description: "Additional metadata for the document (JSON object)".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'operation' parameter".to_string()))?;

        match operation {
            "add_document" => {
                let text = args.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
                    HeliosError::ToolError("Missing 'text' for add_document".to_string())
                })?;

                let metadata: HashMap<String, serde_json::Value> = args
                    .get("metadata")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                let doc_id = self.add_document(text, metadata).await?;
                Ok(ToolResult::success(format!(
                    "✓ Document added successfully\nID: {}\nText preview: {}",
                    doc_id,
                    &text[..text.len().min(100)]
                )))
            }
            "search" => {
                let query = args.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
                    HeliosError::ToolError("Missing 'text' for search".to_string())
                })?;

                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

                let results = self.search(query, limit).await?;

                if results.is_empty() {
                    Ok(ToolResult::success(
                        "No matching documents found".to_string(),
                    ))
                } else {
                    let formatted_results: Vec<String> = results
                        .iter()
                        .enumerate()
                        .map(|(i, (id, score, text))| {
                            format!(
                                "{}. [Score: {:.4}] {}\n   ID: {}\n",
                                i + 1,
                                score,
                                &text[..text.len().min(150)],
                                id
                            )
                        })
                        .collect();

                    Ok(ToolResult::success(format!(
                        "Found {} result(s):\n\n{}",
                        results.len(),
                        formatted_results.join("\n")
                    )))
                }
            }
            "delete" => {
                let doc_id = args.get("doc_id").and_then(|v| v.as_str()).ok_or_else(|| {
                    HeliosError::ToolError("Missing 'doc_id' for delete".to_string())
                })?;

                self.delete_document(doc_id).await?;
                Ok(ToolResult::success(format!(
                    "✓ Document '{}' deleted",
                    doc_id
                )))
            }
            "clear" => {
                self.clear_collection().await?;
                Ok(ToolResult::success(
                    "✓ All documents cleared from collection".to_string(),
                ))
            }
            _ => Err(HeliosError::ToolError(format!(
                "Unknown operation '{}'. Valid: add_document, search, delete, clear",
                operation
            ))),
        }
    }
}

/// In-Memory Database Tool
///
/// Provides a simple key-value store for agents to cache data during conversations.
/// Supports set, get, delete, list keys, and clear operations.
pub struct MemoryDBTool {
    db: std::sync::Arc<std::sync::Mutex<HashMap<String, String>>>,
}

impl MemoryDBTool {
    /// Creates a new `MemoryDBTool`.
    pub fn new() -> Self {
        Self {
            db: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Creates a new `MemoryDBTool` with a shared database.
    pub fn with_shared_db(db: std::sync::Arc<std::sync::Mutex<HashMap<String, String>>>) -> Self {
        Self { db }
    }
}

impl Default for MemoryDBTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for MemoryDBTool {
    fn name(&self) -> &str {
        "memory_db"
    }

    fn description(&self) -> &str {
        "In-memory key-value database for caching data. Operations: set, get, delete, list, clear, exists"
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "operation".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description:
                    "Operation to perform: 'set', 'get', 'delete', 'list', 'clear', 'exists'"
                        .to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "key".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Key for set, get, delete, exists operations".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "value".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Value for set operation".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'operation' parameter".to_string()))?;

        let mut db = self
            .db
            .lock()
            .map_err(|e| HeliosError::ToolError(format!("Failed to lock database: {}", e)))?;

        match operation {
            "set" => {
                let key = args.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
                    HeliosError::ToolError("Missing 'key' parameter for set operation".to_string())
                })?;
                let value = args.get("value").and_then(|v| v.as_str()).ok_or_else(|| {
                    HeliosError::ToolError(
                        "Missing 'value' parameter for set operation".to_string(),
                    )
                })?;

                db.insert(key.to_string(), value.to_string());
                Ok(ToolResult::success(format!(
                    "✓ Set '{}' = '{}'",
                    key, value
                )))
            }
            "get" => {
                let key = args.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
                    HeliosError::ToolError("Missing 'key' parameter for get operation".to_string())
                })?;

                match db.get(key) {
                    Some(value) => Ok(ToolResult::success(format!(
                        "Value for '{}': {}",
                        key, value
                    ))),
                    None => Ok(ToolResult::error(format!("Key '{}' not found", key))),
                }
            }
            "delete" => {
                let key = args.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
                    HeliosError::ToolError(
                        "Missing 'key' parameter for delete operation".to_string(),
                    )
                })?;

                match db.remove(key) {
                    Some(value) => Ok(ToolResult::success(format!(
                        "✓ Deleted '{}' (was: '{}')",
                        key, value
                    ))),
                    None => Ok(ToolResult::error(format!("Key '{}' not found", key))),
                }
            }
            "list" => {
                if db.is_empty() {
                    Ok(ToolResult::success("Database is empty".to_string()))
                } else {
                    let mut items: Vec<String> = db
                        .iter()
                        .map(|(k, v)| format!("  • {} = {}", k, v))
                        .collect();
                    items.sort();
                    Ok(ToolResult::success(format!(
                        "Database contents ({} items):\n{}",
                        db.len(),
                        items.join("\n")
                    )))
                }
            }
            "clear" => {
                let count = db.len();
                db.clear();
                Ok(ToolResult::success(format!(
                    "✓ Cleared database ({} items removed)",
                    count
                )))
            }
            "exists" => {
                let key = args.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
                    HeliosError::ToolError(
                        "Missing 'key' parameter for exists operation".to_string(),
                    )
                })?;

                let exists = db.contains_key(key);
                Ok(ToolResult::success(format!(
                    "Key '{}' exists: {}",
                    key, exists
                )))
            }
            _ => Err(HeliosError::ToolError(format!(
                "Unknown operation '{}'. Valid operations: set, get, delete, list, clear, exists",
                operation
            ))),
        }
    }
}

/// A tool for scraping web content from URLs.
pub struct WebScraperTool;

#[async_trait]
impl Tool for WebScraperTool {
    fn name(&self) -> &str {
        "web_scraper"
    }

    fn description(&self) -> &str {
        "Fetch and extract content from web URLs. Supports HTML text extraction and basic web scraping."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "url".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "The URL to scrape content from".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "extract_text".to_string(),
            ToolParameter {
                param_type: "boolean".to_string(),
                description: "Whether to extract readable text from HTML (default: true)"
                    .to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "timeout_seconds".to_string(),
            ToolParameter {
                param_type: "number".to_string(),
                description: "Request timeout in seconds (default: 30)".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'url' parameter".to_string()))?;

        let extract_text = args
            .get("extract_text")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let timeout_seconds = args
            .get("timeout_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_seconds))
            .user_agent("Helios-WebScraper/1.0")
            .build()
            .map_err(|e| HeliosError::ToolError(format!("Failed to create HTTP client: {}", e)))?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(HeliosError::ToolError(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        let headers = response.headers().clone();
        let content_type = headers
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("");

        let body = response
            .text()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Failed to read response body: {}", e)))?;

        let result = if extract_text && content_type.contains("text/html") {
            // Simple HTML text extraction
            extract_text_from_html(&body)
        } else {
            body
        };

        Ok(ToolResult::success(format!(
            "Content fetched from: {}\nContent-Type: {}\n\n{}",
            url, content_type, result
        )))
    }
}

/// Simple HTML text extraction function.
fn extract_text_from_html(html: &str) -> String {
    // Very basic HTML text extraction - removes tags and keeps text content
    let mut result = String::new();
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    // Clean up whitespace
    result
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// A tool for parsing and manipulating JSON data.
pub struct JsonParserTool;

#[async_trait]
impl Tool for JsonParserTool {
    fn name(&self) -> &str {
        "json_parser"
    }

    fn description(&self) -> &str {
        "Parse, validate, format, and manipulate JSON data. Supports operations: parse, stringify, get_value, set_value, validate"
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "operation".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Operation to perform: 'parse', 'stringify', 'get_value', 'set_value', 'validate'".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "json".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "JSON string for parse/stringify/validate operations".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "path".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description:
                    "JSON path for get_value/set_value operations (e.g., '$.key' or 'key.subkey')"
                        .to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "value".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Value to set for set_value operation (JSON string)".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "indent".to_string(),
            ToolParameter {
                param_type: "number".to_string(),
                description: "Indentation spaces for stringify operation (default: 2)".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'operation' parameter".to_string()))?;

        match operation {
            "parse" => {
                let json_str = args
                    .get("json")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'json' parameter for parse operation".to_string()))?;

                let parsed: Value = serde_json::from_str(json_str)
                    .map_err(|e| HeliosError::ToolError(format!("JSON parse error: {}", e)))?;

                Ok(ToolResult::success(format!(
                    "✓ JSON parsed successfully\nType: {}\nKeys: {}",
                    get_json_type(&parsed),
                    get_json_keys(&parsed)
                )))
            }
            "stringify" => {
                let json_str = args
                    .get("json")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'json' parameter for stringify operation".to_string()))?;

                let parsed: Value = serde_json::from_str(json_str)
                    .map_err(|e| HeliosError::ToolError(format!("Invalid JSON for stringify: {}", e)))?;

                let indent = args
                    .get("indent")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(2) as usize;

                let formatted = if indent == 0 {
                    serde_json::to_string(&parsed)
                } else {
                    serde_json::to_string_pretty(&parsed)
                }
                .map_err(|e| HeliosError::ToolError(format!("JSON stringify error: {}", e)))?;

                Ok(ToolResult::success(formatted))
            }
            "get_value" => {
                let json_str = args
                    .get("json")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'json' parameter for get_value operation".to_string()))?;

                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'path' parameter for get_value operation".to_string()))?;

                let parsed: Value = serde_json::from_str(json_str)
                    .map_err(|e| HeliosError::ToolError(format!("Invalid JSON for get_value: {}", e)))?;

                let value = get_value_by_path(&parsed, path)?;
                Ok(ToolResult::success(format!(
                    "Value at path '{}': {}",
                    path,
                    serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
                )))
            }
            "set_value" => {
                let json_str = args
                    .get("json")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'json' parameter for set_value operation".to_string()))?;

                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'path' parameter for set_value operation".to_string()))?;

                let value_str = args
                    .get("value")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'value' parameter for set_value operation".to_string()))?;

                let new_value: Value = serde_json::from_str(value_str)
                    .map_err(|e| HeliosError::ToolError(format!("Invalid value JSON: {}", e)))?;

                let mut parsed: Value = serde_json::from_str(json_str)
                    .map_err(|e| HeliosError::ToolError(format!("Invalid JSON for set_value: {}", e)))?;

                set_value_by_path(&mut parsed, path, new_value)?;
                let result = serde_json::to_string_pretty(&parsed)
                    .map_err(|e| HeliosError::ToolError(format!("JSON stringify error: {}", e)))?;

                Ok(ToolResult::success(format!(
                    "✓ Value set at path '{}'\n{}",
                    path, result
                )))
            }
            "validate" => {
                let json_str = args
                    .get("json")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'json' parameter for validate operation".to_string()))?;

                match serde_json::from_str::<Value>(json_str) {
                    Ok(_) => Ok(ToolResult::success("✓ JSON is valid".to_string())),
                    Err(e) => Ok(ToolResult::error(format!("✗ JSON validation failed: {}", e))),
                }
            }
            _ => Err(HeliosError::ToolError(format!(
                "Unknown operation '{}'. Valid operations: parse, stringify, get_value, set_value, validate",
                operation
            ))),
        }
    }
}

/// Get the type of a JSON value.
fn get_json_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Get the keys of a JSON object or array length.
fn get_json_keys(value: &Value) -> String {
    match value {
        Value::Object(obj) => {
            let keys: Vec<&String> = obj.keys().collect();
            format!("{{{}}}", keys.len())
        }
        Value::Array(arr) => format!("[{}]", arr.len()),
        _ => "-".to_string(),
    }
}

/// Get a value by JSON path (simple implementation).
fn get_value_by_path(value: &Value, path: &str) -> Result<Value> {
    let path = path.trim_start_matches("$.");
    let keys: Vec<&str> = path.split('.').collect();

    let mut current = value;
    for key in keys {
        match current {
            Value::Object(obj) => {
                current = obj
                    .get(key)
                    .ok_or_else(|| HeliosError::ToolError(format!("Key '{}' not found", key)))?;
            }
            _ => {
                return Err(HeliosError::ToolError(format!(
                    "Cannot access '{}' on non-object",
                    key
                )))
            }
        }
    }

    Ok(current.clone())
}

/// Set a value by JSON path (simple implementation).
fn set_value_by_path(value: &mut Value, path: &str, new_value: Value) -> Result<()> {
    let path = path.trim_start_matches("$.");
    let keys: Vec<&str> = path.split('.').collect();

    if keys.is_empty() {
        return Err(HeliosError::ToolError("Empty path".to_string()));
    }

    let mut current = value;
    for (i, key) in keys.iter().enumerate() {
        if i == keys.len() - 1 {
            // Last key - set the value
            match current {
                Value::Object(obj) => {
                    obj.insert(key.to_string(), new_value);
                    return Ok(());
                }
                _ => {
                    return Err(HeliosError::ToolError(format!(
                        "Cannot set '{}' on non-object",
                        key
                    )))
                }
            }
        } else {
            // Intermediate key - navigate deeper
            match current {
                Value::Object(obj) => {
                    if !obj.contains_key(*key) {
                        obj.insert(key.to_string(), Value::Object(serde_json::Map::new()));
                    }
                    current = obj.get_mut(*key).unwrap();
                }
                _ => {
                    return Err(HeliosError::ToolError(format!(
                        "Cannot access '{}' on non-object",
                        key
                    )))
                }
            }
        }
    }

    Ok(())
}

/// A tool for date/time operations and timestamp manipulation.
pub struct TimestampTool;

#[async_trait]
impl Tool for TimestampTool {
    fn name(&self) -> &str {
        "timestamp"
    }

    fn description(&self) -> &str {
        "Work with timestamps and date/time operations. Supports current time, formatting, parsing, and time arithmetic."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "operation".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Operation: 'now', 'format', 'parse', 'add', 'subtract', 'diff'"
                    .to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "timestamp".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Timestamp string for parse/format operations".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "format".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Date format string (default: RFC3339)".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "unit".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Time unit for arithmetic: 'seconds', 'minutes', 'hours', 'days'"
                    .to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "amount".to_string(),
            ToolParameter {
                param_type: "number".to_string(),
                description: "Amount for add/subtract operations".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "timestamp1".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "First timestamp for diff operation".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "timestamp2".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Second timestamp for diff operation".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'operation' parameter".to_string()))?;

        match operation {
            "now" => {
                let now = chrono::Utc::now();
                let timestamp = now.timestamp();
                let rfc3339 = now.to_rfc3339();

                Ok(ToolResult::success(format!(
                    "Current time:\nUnix timestamp: {}\nRFC3339: {}\nLocal: {}",
                    timestamp,
                    rfc3339,
                    now.with_timezone(&chrono::Local::now().timezone())
                )))
            }
            "format" => {
                let timestamp_str =
                    args.get("timestamp")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            HeliosError::ToolError(
                                "Missing 'timestamp' parameter for format operation".to_string(),
                            )
                        })?;

                let format_str = args
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("%Y-%m-%d %H:%M:%S");

                // Try parsing as unix timestamp first, then as RFC3339
                let dt = if let Ok(ts) = timestamp_str.parse::<i64>() {
                    chrono::DateTime::from_timestamp(ts, 0).ok_or_else(|| {
                        HeliosError::ToolError("Invalid unix timestamp".to_string())
                    })?
                } else {
                    chrono::DateTime::parse_from_rfc3339(timestamp_str)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .or_else(|_| {
                            chrono::NaiveDateTime::parse_from_str(timestamp_str, format_str)
                                .map(|ndt| ndt.and_utc())
                                .map_err(|e| {
                                    HeliosError::ToolError(format!(
                                        "Failed to parse timestamp: {}",
                                        e
                                    ))
                                })
                        })
                        .map_err(|e| {
                            HeliosError::ToolError(format!("Failed to parse timestamp: {}", e))
                        })?
                };

                let formatted = dt.format(format_str).to_string();
                Ok(ToolResult::success(format!(
                    "Formatted timestamp: {}",
                    formatted
                )))
            }
            "parse" => {
                let timestamp_str =
                    args.get("timestamp")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            HeliosError::ToolError(
                                "Missing 'timestamp' parameter for parse operation".to_string(),
                            )
                        })?;

                let format_str = args
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("%Y-%m-%d %H:%M:%S");

                // Try multiple parsing strategies
                let dt = chrono::DateTime::parse_from_rfc3339(timestamp_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .or_else(|_| {
                        chrono::NaiveDateTime::parse_from_str(timestamp_str, format_str)
                            .map(|ndt| ndt.and_utc())
                    })
                    .map_err(|e| {
                        HeliosError::ToolError(format!("Failed to parse timestamp: {}", e))
                    })?;

                let unix_ts = dt.timestamp();
                let rfc3339 = dt.to_rfc3339();

                Ok(ToolResult::success(format!(
                    "Parsed timestamp:\nUnix: {}\nRFC3339: {}\nFormatted: {}",
                    unix_ts,
                    rfc3339,
                    dt.format("%Y-%m-%d %H:%M:%S UTC")
                )))
            }
            "add" | "subtract" => {
                let default_timestamp = chrono::Utc::now().to_rfc3339();
                let timestamp_str = args
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&default_timestamp);

                let unit = args.get("unit").and_then(|v| v.as_str()).ok_or_else(|| {
                    HeliosError::ToolError(
                        "Missing 'unit' parameter for arithmetic operation".to_string(),
                    )
                })?;

                let amount = args.get("amount").and_then(|v| v.as_i64()).ok_or_else(|| {
                    HeliosError::ToolError(
                        "Missing 'amount' parameter for arithmetic operation".to_string(),
                    )
                })?;

                let dt = chrono::DateTime::parse_from_rfc3339(timestamp_str)
                    .or_else(|_| {
                        if let Ok(ts) = timestamp_str.parse::<i64>() {
                            chrono::DateTime::from_timestamp(ts, 0)
                                .ok_or_else(|| {
                                    HeliosError::ToolError("Invalid unix timestamp".to_string())
                                })
                                .map(|dt| dt.into())
                        } else {
                            Err(HeliosError::ToolError(
                                "Invalid timestamp format".to_string(),
                            ))
                        }
                    })
                    .map_err(|e| {
                        HeliosError::ToolError(format!("Failed to parse timestamp: {}", e))
                    })?;

                let duration = match unit {
                    "seconds" => chrono::Duration::seconds(amount),
                    "minutes" => chrono::Duration::minutes(amount),
                    "hours" => chrono::Duration::hours(amount),
                    "days" => chrono::Duration::days(amount),
                    _ => {
                        return Err(HeliosError::ToolError(format!(
                            "Unknown unit '{}'. Use: seconds, minutes, hours, days",
                            unit
                        )))
                    }
                };

                let result_dt = if operation == "add" {
                    dt + duration
                } else {
                    dt - duration
                };

                Ok(ToolResult::success(format!(
                    "{} {} {} to {}\nResult: {}\nUnix: {}",
                    if operation == "add" {
                        "Added"
                    } else {
                        "Subtracted"
                    },
                    amount.abs(),
                    unit,
                    timestamp_str,
                    result_dt.to_rfc3339(),
                    result_dt.timestamp()
                )))
            }
            "diff" => {
                let ts1_str = args
                    .get("timestamp1")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        HeliosError::ToolError(
                            "Missing 'timestamp1' parameter for diff operation".to_string(),
                        )
                    })?;

                let ts2_str = args
                    .get("timestamp2")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        HeliosError::ToolError(
                            "Missing 'timestamp2' parameter for diff operation".to_string(),
                        )
                    })?;

                let dt1 = parse_timestamp(ts1_str)?;
                let dt2 = parse_timestamp(ts2_str)?;

                let duration = if dt1 > dt2 { dt1 - dt2 } else { dt2 - dt1 };
                let seconds = duration.num_seconds();
                let minutes = duration.num_minutes();
                let hours = duration.num_hours();
                let days = duration.num_days();

                Ok(ToolResult::success(format!(
                    "Time difference between {} and {}:\n{} seconds\n{} minutes\n{} hours\n{} days",
                    ts1_str, ts2_str, seconds, minutes, hours, days
                )))
            }
            _ => Err(HeliosError::ToolError(format!(
                "Unknown operation '{}'. Valid operations: now, format, parse, add, subtract, diff",
                operation
            ))),
        }
    }
}

/// Parse a timestamp string using multiple strategies.
fn parse_timestamp(ts_str: &str) -> Result<chrono::DateTime<chrono::Utc>> {
    if let Ok(ts) = ts_str.parse::<i64>() {
        chrono::DateTime::from_timestamp(ts, 0)
            .ok_or_else(|| HeliosError::ToolError("Invalid unix timestamp".to_string()))
    } else {
        chrono::DateTime::parse_from_rfc3339(ts_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|_| HeliosError::ToolError("Invalid timestamp format".to_string()))
    }
}

/// A tool for basic file I/O operations.
pub struct FileIOTool;

#[async_trait]
impl Tool for FileIOTool {
    fn name(&self) -> &str {
        "file_io"
    }

    fn description(&self) -> &str {
        "Basic file operations: read, write, append, delete, copy, move. Unified interface for common file I/O tasks."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "operation".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Operation: 'read', 'write', 'append', 'delete', 'copy', 'move', 'exists', 'size'".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "path".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "File path for operations".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "src_path".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Source path for copy/move operations".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "dst_path".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Destination path for copy/move operations".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "content".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Content for write/append operations".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'operation' parameter".to_string()))?;

        match operation {
            "read" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'path' parameter for read operation".to_string()))?;

                let content = std::fs::read_to_string(path)
                    .map_err(|e| HeliosError::ToolError(format!("Failed to read file: {}", e)))?;

                Ok(ToolResult::success(format!(
                    "File: {}\nSize: {} bytes\n\n{}",
                    path,
                    content.len(),
                    content
                )))
            }
            "write" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'path' parameter for write operation".to_string()))?;

                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'content' parameter for write operation".to_string()))?;

                // Create parent directories if they don't exist
                if let Some(parent) = std::path::Path::new(path).parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        HeliosError::ToolError(format!("Failed to create directories: {}", e))
                    })?;
                }

                std::fs::write(path, content)
                    .map_err(|e| HeliosError::ToolError(format!("Failed to write file: {}", e)))?;

                Ok(ToolResult::success(format!(
                    "✓ Wrote {} bytes to {}",
                    content.len(),
                    path
                )))
            }
            "append" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'path' parameter for append operation".to_string()))?;

                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'content' parameter for append operation".to_string()))?;

                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .and_then(|mut file| std::io::Write::write_all(&mut file, content.as_bytes()))
                    .map_err(|e| HeliosError::ToolError(format!("Failed to append to file: {}", e)))?;

                Ok(ToolResult::success(format!(
                    "✓ Appended {} bytes to {}",
                    content.len(),
                    path
                )))
            }
            "delete" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'path' parameter for delete operation".to_string()))?;

                let metadata = std::fs::metadata(path)
                    .map_err(|e| HeliosError::ToolError(format!("Cannot access file: {}", e)))?;

                let file_type = if metadata.is_file() { "file" } else { "directory" };

                if metadata.is_file() {
                    std::fs::remove_file(path)
                        .map_err(|e| HeliosError::ToolError(format!("Failed to delete file: {}", e)))?;
                } else {
                    std::fs::remove_dir_all(path)
                        .map_err(|e| HeliosError::ToolError(format!("Failed to delete directory: {}", e)))?;
                }

                Ok(ToolResult::success(format!(
                    "✓ Deleted {}: {}",
                    file_type, path
                )))
            }
            "copy" => {
                let src_path = args
                    .get("src_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'src_path' parameter for copy operation".to_string()))?;

                let dst_path = args
                    .get("dst_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'dst_path' parameter for copy operation".to_string()))?;

                std::fs::copy(src_path, dst_path)
                    .map_err(|e| HeliosError::ToolError(format!("Failed to copy file: {}", e)))?;

                Ok(ToolResult::success(format!(
                    "✓ Copied {} to {}",
                    src_path, dst_path
                )))
            }
            "move" => {
                let src_path = args
                    .get("src_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'src_path' parameter for move operation".to_string()))?;

                let dst_path = args
                    .get("dst_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'dst_path' parameter for move operation".to_string()))?;

                std::fs::rename(src_path, dst_path)
                    .map_err(|e| HeliosError::ToolError(format!("Failed to move file: {}", e)))?;

                Ok(ToolResult::success(format!(
                    "✓ Moved {} to {}",
                    src_path, dst_path
                )))
            }
            "exists" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'path' parameter for exists operation".to_string()))?;

                let exists = std::path::Path::new(path).exists();
                let file_type = if exists {
                    if std::fs::metadata(path).map(|m| m.is_file()).unwrap_or(false) {
                        "file"
                    } else {
                        "directory"
                    }
                } else {
                    "nonexistent"
                };

                Ok(ToolResult::success(format!(
                    "Path '{}' exists: {} ({})",
                    path, exists, file_type
                )))
            }
            "size" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'path' parameter for size operation".to_string()))?;

                let metadata = std::fs::metadata(path)
                    .map_err(|e| HeliosError::ToolError(format!("Cannot access file: {}", e)))?;

                let size = metadata.len();
                Ok(ToolResult::success(format!(
                    "Size of '{}': {} bytes",
                    path, size
                )))
            }
            _ => Err(HeliosError::ToolError(format!(
                "Unknown operation '{}'. Valid operations: read, write, append, delete, copy, move, exists, size",
                operation
            ))),
        }
    }
}

/// A tool for executing shell commands safely.
pub struct ShellCommandTool;

#[async_trait]
impl Tool for ShellCommandTool {
    fn name(&self) -> &str {
        "shell_command"
    }

    fn description(&self) -> &str {
        "Execute shell commands with safety restrictions. Limited to basic commands, no destructive operations allowed."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "command".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Shell command to execute".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "timeout_seconds".to_string(),
            ToolParameter {
                param_type: "number".to_string(),
                description: "Command timeout in seconds (default: 30, max: 60)".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'command' parameter".to_string()))?;

        let timeout_seconds = args
            .get("timeout_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(30)
            .min(60); // Max 60 seconds

        // Safety check - block dangerous commands
        let dangerous_patterns = [
            "rm ",
            "rmdir",
            "del ",
            "format",
            "fdisk",
            "mkfs",
            "dd ",
            "shred",
            "wipe",
            "sudo",
            "su ",
            "chmod 777",
            "chown root",
            "passwd",
            "usermod",
            "userdel",
            ">",
            ">>",
            "|",
            ";",
            "&&",
            "||",
            "`",
            "$(",
        ];

        for pattern in &dangerous_patterns {
            if command.contains(pattern) {
                return Err(HeliosError::ToolError(format!(
                    "Command blocked for safety: contains '{}'",
                    pattern
                )));
            }
        }

        // Execute command with timeout
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_seconds),
            tokio::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .output(),
        )
        .await
        .map_err(|_| {
            HeliosError::ToolError(format!(
                "Command timed out after {} seconds",
                timeout_seconds
            ))
        })?
        .map_err(|e| HeliosError::ToolError(format!("Failed to execute command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let exit_code = output.status.code().unwrap_or(-1);

        let mut result = format!("Command: {}\nExit code: {}\n", command, exit_code);

        if !stdout.is_empty() {
            result.push_str(&format!("Stdout:\n{}\n", stdout));
        }

        if !stderr.is_empty() {
            result.push_str(&format!("Stderr:\n{}\n", stderr));
        }

        if exit_code == 0 {
            Ok(ToolResult::success(result))
        } else {
            Ok(ToolResult::error(result))
        }
    }
}

/// A tool for making HTTP requests.
pub struct HttpRequestTool;

#[async_trait]
impl Tool for HttpRequestTool {
    fn name(&self) -> &str {
        "http_request"
    }

    fn description(&self) -> &str {
        "Make HTTP requests with various methods. Supports GET, POST, PUT, DELETE with custom headers and body."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "method".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "HTTP method: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS"
                    .to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "url".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Request URL".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "headers".to_string(),
            ToolParameter {
                param_type: "object".to_string(),
                description: "Request headers as JSON object (optional)".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "body".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Request body for POST/PUT/PATCH methods".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "timeout_seconds".to_string(),
            ToolParameter {
                param_type: "number".to_string(),
                description: "Request timeout in seconds (default: 30)".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let method = args
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'method' parameter".to_string()))?;

        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'url' parameter".to_string()))?;

        let timeout_seconds = args
            .get("timeout_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_seconds))
            .build()
            .map_err(|e| HeliosError::ToolError(format!("Failed to create HTTP client: {}", e)))?;

        let mut request = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "PATCH" => client.patch(url),
            "HEAD" => client.head(url),
            _ => {
                return Err(HeliosError::ToolError(format!(
                    "Unsupported HTTP method: {}",
                    method
                )))
            }
        };

        // Add headers
        if let Some(headers) = args.get("headers") {
            if let Some(headers_obj) = headers.as_object() {
                for (key, value) in headers_obj {
                    if let Some(value_str) = value.as_str() {
                        request = request.header(key, value_str);
                    }
                }
            }
        }

        // Add body for methods that support it
        if matches!(method.to_uppercase().as_str(), "POST" | "PUT" | "PATCH") {
            if let Some(body) = args.get("body").and_then(|v| v.as_str()) {
                request = request.body(body.to_string());
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("HTTP request failed: {}", e)))?;

        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Binary content".to_string());

        let mut result = format!(
            "HTTP {} {}\nStatus: {}\n\n",
            method.to_uppercase(),
            url,
            status
        );

        // Add response headers
        result.push_str("Response Headers:\n");
        for (name, value) in headers.iter() {
            if let Ok(value_str) = value.to_str() {
                result.push_str(&format!("{}: {}\n", name, value_str));
            }
        }
        result.push_str("\nResponse Body:\n");
        result.push_str(&body);

        if status.is_success() {
            Ok(ToolResult::success(result))
        } else {
            Ok(ToolResult::error(result))
        }
    }
}

/// A tool for listing directory contents.
pub struct FileListTool;

#[async_trait]
impl Tool for FileListTool {
    fn name(&self) -> &str {
        "file_list"
    }

    fn description(&self) -> &str {
        "List directory contents with detailed information including file sizes, types, and modification times."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "path".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Directory path to list (default: current directory)".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "show_hidden".to_string(),
            ToolParameter {
                param_type: "boolean".to_string(),
                description: "Show hidden files/directories (default: false)".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "recursive".to_string(),
            ToolParameter {
                param_type: "boolean".to_string(),
                description: "List contents recursively (default: false)".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "max_depth".to_string(),
            ToolParameter {
                param_type: "number".to_string(),
                description: "Maximum recursion depth (default: 3)".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let base_path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let show_hidden = args
            .get("show_hidden")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let recursive = args
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let max_depth = args.get("max_depth").and_then(|v| v.as_u64()).unwrap_or(3) as usize;

        let mut results = Vec::new();

        if recursive {
            for entry in walkdir::WalkDir::new(base_path)
                .max_depth(max_depth)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if let Some(entry_info) = format_walkdir_entry(&entry, show_hidden) {
                    results.push(entry_info);
                }
            }
        } else {
            let entries = std::fs::read_dir(base_path)
                .map_err(|e| HeliosError::ToolError(format!("Failed to read directory: {}", e)))?;

            for entry in entries.filter_map(|e| e.ok()) {
                if let Some(entry_info) = format_entry(&entry, show_hidden) {
                    results.push(entry_info);
                }
            }
        }

        results.sort();

        let mut output = format!("Directory listing for: {}\n\n", base_path);
        output.push_str(&format!("Total items: {}\n\n", results.len()));

        for entry in results {
            output.push_str(&entry);
            output.push('\n');
        }

        Ok(ToolResult::success(output))
    }
}

/// Format a walkdir directory entry with metadata.
fn format_walkdir_entry(entry: &walkdir::DirEntry, show_hidden: bool) -> Option<String> {
    let path = entry.path();
    let file_name = path.file_name()?.to_str()?;

    // Skip hidden files if not requested
    if !show_hidden && file_name.starts_with('.') {
        return None;
    }

    let metadata = entry.metadata().ok()?;

    let file_type = if metadata.is_dir() { "DIR" } else { "FILE" };
    let size = metadata.len();
    let modified = metadata.modified().ok()?;
    let modified_dt = chrono::DateTime::<chrono::Local>::from(modified);
    let modified_str = modified_dt.format("%Y-%m-%d %H:%M:%S").to_string();

    Some(format!(
        "{:4} {:>8} {} {}",
        file_type,
        size,
        modified_str,
        path.display()
    ))
}

/// Format a directory entry with metadata.
fn format_entry(entry: &std::fs::DirEntry, show_hidden: bool) -> Option<String> {
    let path = entry.path();
    let file_name = path.file_name()?.to_str()?;

    // Skip hidden files if not requested
    if !show_hidden && file_name.starts_with('.') {
        return None;
    }

    let metadata = entry.metadata().ok()?;

    let file_type = if metadata.is_dir() { "DIR" } else { "FILE" };
    let size = metadata.len();
    let modified = metadata.modified().ok()?;
    let modified_dt = chrono::DateTime::<chrono::Local>::from(modified);
    let modified_str = modified_dt.format("%Y-%m-%d %H:%M:%S").to_string();

    Some(format!(
        "{:4} {:>8} {} {}",
        file_type,
        size,
        modified_str,
        path.display()
    ))
}

/// A tool for retrieving system information.
pub struct SystemInfoTool;

#[async_trait]
impl Tool for SystemInfoTool {
    fn name(&self) -> &str {
        "system_info"
    }

    fn description(&self) -> &str {
        "Retrieve system information including OS, CPU, memory, disk usage, and network interfaces."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "category".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description:
                    "Info category: 'all', 'os', 'cpu', 'memory', 'disk', 'network' (default: all)"
                        .to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let category = args
            .get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("all");

        let mut system = sysinfo::System::new_all();
        system.refresh_all();

        let disks = sysinfo::Disks::new_with_refreshed_list();
        let networks = sysinfo::Networks::new_with_refreshed_list();

        let mut output = String::new();

        match category {
            "all" => {
                output.push_str(&get_os_info(&system));
                output.push_str(&get_cpu_info(&system));
                output.push_str(&get_memory_info(&system));
                output.push_str(&get_disk_info(&disks));
                output.push_str(&get_network_info(&networks));
            }
            "os" => output.push_str(&get_os_info(&system)),
            "cpu" => output.push_str(&get_cpu_info(&system)),
            "memory" => output.push_str(&get_memory_info(&system)),
            "disk" => output.push_str(&get_disk_info(&disks)),
            "network" => output.push_str(&get_network_info(&networks)),
            _ => {
                return Err(HeliosError::ToolError(format!(
                    "Unknown category '{}'. Use: all, os, cpu, memory, disk, network",
                    category
                )))
            }
        }

        Ok(ToolResult::success(output))
    }
}

/// Get operating system information.
fn get_os_info(_system: &sysinfo::System) -> String {
    let mut info = String::from("=== Operating System ===\n");

    info.push_str(&format!("OS: {}\n", std::env::consts::OS));
    info.push_str(&format!("Architecture: {}\n", std::env::consts::ARCH));
    info.push_str(&format!("Family: {}\n", std::env::consts::FAMILY));

    if let Ok(hostname) = hostname::get() {
        if let Some(hostname_str) = hostname.to_str() {
            info.push_str(&format!("Hostname: {}\n", hostname_str));
        }
    }

    info.push_str(&format!("Uptime: {} seconds\n", sysinfo::System::uptime()));
    info.push('\n');

    info
}

/// Get CPU information.
fn get_cpu_info(system: &sysinfo::System) -> String {
    let mut info = String::from("=== CPU Information ===\n");

    info.push_str(&format!(
        "Physical cores: {}\n",
        system.physical_core_count().unwrap_or(0)
    ));
    info.push_str(&format!("Logical cores: {}\n", system.cpus().len()));

    for (i, cpu) in system.cpus().iter().enumerate() {
        if i >= 4 {
            // Limit output to first 4 CPUs
            info.push_str("... and more CPUs\n");
            break;
        }
        info.push_str(&format!("CPU {}: {:.1}% usage\n", i, cpu.cpu_usage()));
    }

    info.push('\n');
    info
}

/// Get memory information.
fn get_memory_info(system: &sysinfo::System) -> String {
    let mut info = String::from("=== Memory Information ===\n");

    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    let available_memory = system.available_memory();

    info.push_str(&format!(
        "Total memory: {} MB\n",
        total_memory / 1024 / 1024
    ));
    info.push_str(&format!("Used memory: {} MB\n", used_memory / 1024 / 1024));
    info.push_str(&format!(
        "Available memory: {} MB\n",
        available_memory / 1024 / 1024
    ));
    info.push_str(&format!(
        "Memory usage: {:.1}%\n",
        (used_memory as f64 / total_memory as f64) * 100.0
    ));

    info.push('\n');
    info
}

/// Get disk information.
fn get_disk_info(disks: &sysinfo::Disks) -> String {
    let mut info = String::from("=== Disk Information ===\n");

    for disk in disks.list() {
        let total_space = disk.total_space();
        let available_space = disk.available_space();
        let used_space = total_space - available_space;

        info.push_str(&format!("Mount point: {}\n", disk.mount_point().display()));
        info.push_str(&format!(
            "File system: {}\n",
            disk.file_system().to_string_lossy()
        ));
        info.push_str(&format!(
            "Total space: {} GB\n",
            total_space / 1024 / 1024 / 1024
        ));
        info.push_str(&format!(
            "Used space: {} GB\n",
            used_space / 1024 / 1024 / 1024
        ));
        info.push_str(&format!(
            "Available space: {} GB\n",
            available_space / 1024 / 1024 / 1024
        ));
        info.push_str(&format!(
            "Usage: {:.1}%\n\n",
            (used_space as f64 / total_space as f64) * 100.0
        ));
    }

    info
}

/// Get network information.
fn get_network_info(networks: &sysinfo::Networks) -> String {
    let mut info = String::from("=== Network Information ===\n");

    for (interface_name, data) in networks.list() {
        info.push_str(&format!("Interface: {}\n", interface_name));

        // MAC address and IP information - simplified due to API changes
        info.push_str(&format!("Received: {} bytes\n", data.received()));
        info.push_str(&format!("Transmitted: {} bytes\n", data.transmitted()));
        info.push('\n');
    }

    info
}

/// A tool for text processing and manipulation operations.
pub struct TextProcessorTool;

#[async_trait]
impl Tool for TextProcessorTool {
    fn name(&self) -> &str {
        "text_processor"
    }

    fn description(&self) -> &str {
        "Process and manipulate text with operations like search, replace, split, join, count, and format."
    }

    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "operation".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Operation: 'search', 'replace', 'split', 'join', 'count', 'uppercase', 'lowercase', 'trim', 'lines', 'words'".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "text".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Input text for processing".to_string(),
                required: Some(true),
            },
        );
        params.insert(
            "pattern".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Search pattern for search/replace/split operations".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "replacement".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Replacement text for replace operation".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "separator".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Separator for join/split operations (default: space)".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "case_sensitive".to_string(),
            ToolParameter {
                param_type: "boolean".to_string(),
                description: "Case sensitive search (default: true)".to_string(),
                required: Some(false),
            },
        );
        params
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'operation' parameter".to_string()))?;

        let text = args
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HeliosError::ToolError("Missing 'text' parameter".to_string()))?;

        match operation {
            "search" => {
                let pattern = args
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'pattern' parameter for search operation".to_string()))?;

                let case_sensitive = args.get("case_sensitive").and_then(|v| v.as_bool()).unwrap_or(true);

                let regex = if case_sensitive {
                    regex::Regex::new(pattern)
                } else {
                    regex::RegexBuilder::new(pattern).case_insensitive(true).build()
                }
                .map_err(|e| HeliosError::ToolError(format!("Invalid regex pattern: {}", e)))?;

                let matches: Vec<(usize, &str)> = regex
                    .find_iter(text)
                    .map(|m| (m.start(), m.as_str()))
                    .collect();

                let result = if matches.is_empty() {
                    "No matches found".to_string()
                } else {
                    let mut output = format!("Found {} match(es):\n", matches.len());
                    for (i, (pos, match_text)) in matches.iter().enumerate() {
                        output.push_str(&format!("{}. Position {}: '{}'\n", i + 1, pos, match_text));
                    }
                    output
                };

                Ok(ToolResult::success(result))
            }
            "replace" => {
                let pattern = args
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'pattern' parameter for replace operation".to_string()))?;

                let replacement = args
                    .get("replacement")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let case_sensitive = args.get("case_sensitive").and_then(|v| v.as_bool()).unwrap_or(true);

                let regex = if case_sensitive {
                    regex::Regex::new(pattern)
                } else {
                    regex::RegexBuilder::new(pattern).case_insensitive(true).build()
                }
                .map_err(|e| HeliosError::ToolError(format!("Invalid regex pattern: {}", e)))?;

                let result = regex.replace_all(text, replacement).to_string();
                let count = regex.find_iter(text).count();

                Ok(ToolResult::success(format!(
                    "Replaced {} occurrence(s):\n\n{}",
                    count, result
                )))
            }
            "split" => {
                let separator = args
                    .get("separator")
                    .and_then(|v| v.as_str())
                    .unwrap_or(" ");

                let parts: Vec<&str> = text.split(separator).collect();
                let result = format!(
                    "Split into {} parts:\n{}",
                    parts.len(),
                    parts.iter().enumerate()
                        .map(|(i, part)| format!("{}. '{}'", i + 1, part))
                        .collect::<Vec<_>>()
                        .join("\n")
                );

                Ok(ToolResult::success(result))
            }
            "join" => {
                let separator = args
                    .get("separator")
                    .and_then(|v| v.as_str())
                    .unwrap_or(" ");

                let lines: Vec<&str> = text.lines().collect();
                let result = lines.join(separator);

                Ok(ToolResult::success(format!(
                    "Joined {} lines with '{}':\n{}",
                    lines.len(), separator, result
                )))
            }
            "count" => {
                let chars = text.chars().count();
                let bytes = text.len();
                let lines = text.lines().count();
                let words = text.split_whitespace().count();

                Ok(ToolResult::success(format!(
                    "Text statistics:\nCharacters: {}\nBytes: {}\nLines: {}\nWords: {}",
                    chars, bytes, lines, words
                )))
            }
            "uppercase" => {
                Ok(ToolResult::success(text.to_uppercase()))
            }
            "lowercase" => {
                Ok(ToolResult::success(text.to_lowercase()))
            }
            "trim" => {
                Ok(ToolResult::success(text.trim().to_string()))
            }
            "lines" => {
                let lines: Vec<String> = text.lines()
                    .enumerate()
                    .map(|(i, line)| format!("{:4}: {}", i + 1, line))
                    .collect();

                Ok(ToolResult::success(format!(
                    "Text with line numbers ({} lines):\n{}",
                    lines.len(),
                    lines.join("\n")
                )))
            }
            "words" => {
                let words: Vec<String> = text.split_whitespace()
                    .enumerate()
                    .map(|(i, word)| format!("{:4}: {}", i + 1, word))
                    .collect();

                Ok(ToolResult::success(format!(
                    "Words ({} total):\n{}",
                    words.len(),
                    words.iter()
                        .collect::<Vec<_>>()
                        .chunks(10)
                        .enumerate()
                        .map(|(chunk_i, chunk)| {
                            format!("Line {}: {}", chunk_i + 1,
                                chunk.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" "))
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                )))
            }
            _ => Err(HeliosError::ToolError(format!(
                "Unknown operation '{}'. Valid operations: search, replace, split, join, count, uppercase, lowercase, trim, lines, words",
                operation
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Tests the creation of a successful `ToolResult`.
    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success("test output");
        assert!(result.success);
        assert_eq!(result.output, "test output");
    }

    /// Tests the file search tool with a glob pattern.
    #[tokio::test]
    async fn test_file_search_tool_glob_pattern_precompiled_regex() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let base_tmp = std::env::temp_dir();
        let pid = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_dir = base_tmp.join(format!("helios_fs_test_{}_{}", pid, nanos));
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create files
        let file_rs = test_dir.join("a.rs");
        let file_txt = test_dir.join("b.txt");
        let subdir = test_dir.join("subdir");
        std::fs::create_dir_all(&subdir).unwrap();
        let file_sub_rs = subdir.join("mod.rs");
        std::fs::write(&file_rs, "fn main() {}\n").unwrap();
        std::fs::write(&file_txt, "hello\n").unwrap();
        std::fs::write(&file_sub_rs, "pub fn x() {}\n").unwrap();

        // Execute search with glob pattern
        let tool = FileSearchTool;
        let args = json!({
            "path": test_dir.to_string_lossy(),
            "pattern": "*.rs",
            "max_results": 50
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        let out = result.output;
        // Should find .rs files
        assert!(out.contains(&file_rs.to_string_lossy().to_string()));
        assert!(out.contains(&file_sub_rs.to_string_lossy().to_string()));
        // Should not include .txt
        assert!(!out.contains(&file_txt.to_string_lossy().to_string()));

        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    /// Tests the file search tool with an invalid pattern.
    #[tokio::test]
    async fn test_file_search_tool_invalid_pattern_fallback_contains() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let base_tmp = std::env::temp_dir();
        let pid = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_dir = base_tmp.join(format!("helios_fs_test_invalid_{}_{}", pid, nanos));
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create file with '(' to be matched by substring fallback
        let special = test_dir.join("foo(bar).txt");
        std::fs::write(&special, "content\n").unwrap();

        let tool = FileSearchTool;
        let args = json!({
            "path": test_dir.to_string_lossy(),
            "pattern": "(",
            "max_results": 50
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        let out = result.output;
        assert!(out.contains(&special.to_string_lossy().to_string()));

        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    /// Tests the creation of an error `ToolResult`.
    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("test error");
        assert!(!result.success);
        assert_eq!(result.output, "test error");
    }

    /// Tests the calculator tool.
    #[tokio::test]
    async fn test_calculator_tool() {
        let tool = CalculatorTool;
        assert_eq!(tool.name(), "calculator");
        assert_eq!(
            tool.description(),
            "Perform basic arithmetic operations. Supports +, -, *, / operations."
        );

        let args = json!({"expression": "2 + 2"});
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "4");
    }

    /// Tests the calculator tool with multiplication.
    #[tokio::test]
    async fn test_calculator_tool_multiplication() {
        let tool = CalculatorTool;
        let args = json!({"expression": "3 * 4"});
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "12");
    }

    /// Tests the calculator tool with division.
    #[tokio::test]
    async fn test_calculator_tool_division() {
        let tool = CalculatorTool;
        let args = json!({"expression": "8 / 2"});
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "4");
    }

    /// Tests the calculator tool with division by zero.
    #[tokio::test]
    async fn test_calculator_tool_division_by_zero() {
        let tool = CalculatorTool;
        let args = json!({"expression": "8 / 0"});
        let result = tool.execute(args).await;
        assert!(result.is_err());
    }

    /// Tests the calculator tool with an invalid expression.
    #[tokio::test]
    async fn test_calculator_tool_invalid_expression() {
        let tool = CalculatorTool;
        let args = json!({"expression": "invalid"});
        let result = tool.execute(args).await;
        assert!(result.is_err());
    }

    /// Tests the echo tool.
    #[tokio::test]
    async fn test_echo_tool() {
        let tool = EchoTool;
        assert_eq!(tool.name(), "echo");
        assert_eq!(tool.description(), "Echo back the provided message.");

        let args = json!({"message": "Hello, world!"});
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "Echo: Hello, world!");
    }

    /// Tests the echo tool with a missing parameter.
    #[tokio::test]
    async fn test_echo_tool_missing_parameter() {
        let tool = EchoTool;
        let args = json!({});
        let result = tool.execute(args).await;
        assert!(result.is_err());
    }

    /// Tests the creation of a new `ToolRegistry`.
    #[test]
    fn test_tool_registry_new() {
        let registry = ToolRegistry::new();
        assert!(registry.tools.is_empty());
    }

    /// Tests registering and getting a tool from the `ToolRegistry`.
    #[tokio::test]
    async fn test_tool_registry_register_and_get() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(CalculatorTool));

        let tool = registry.get("calculator");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "calculator");
    }

    /// Tests executing a tool from the `ToolRegistry`.
    #[tokio::test]
    async fn test_tool_registry_execute() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(CalculatorTool));

        let args = json!({"expression": "5 * 6"});
        let result = registry.execute("calculator", args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "30");
    }

    /// Tests executing a nonexistent tool from the `ToolRegistry`.
    #[tokio::test]
    async fn test_tool_registry_execute_nonexistent_tool() {
        let registry = ToolRegistry::new();
        let args = json!({"expression": "5 * 6"});
        let result = registry.execute("nonexistent", args).await;
        assert!(result.is_err());
    }

    /// Tests getting the definitions of all tools in the `ToolRegistry`.
    #[test]
    fn test_tool_registry_get_definitions() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(CalculatorTool));
        registry.register(Box::new(EchoTool));

        let definitions = registry.get_definitions();
        assert_eq!(definitions.len(), 2);

        // Check that we have both tools
        let names: Vec<String> = definitions
            .iter()
            .map(|d| d.function.name.clone())
            .collect();
        assert!(names.contains(&"calculator".to_string()));
        assert!(names.contains(&"echo".to_string()));
    }

    /// Tests listing the names of all tools in the `ToolRegistry`.
    #[test]
    fn test_tool_registry_list_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(CalculatorTool));
        registry.register(Box::new(EchoTool));

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 2);
        assert!(tools.contains(&"calculator".to_string()));
        assert!(tools.contains(&"echo".to_string()));
    }

    /// Tests setting and getting a value in the `MemoryDBTool`.
    #[tokio::test]
    async fn test_memory_db_set_and_get() {
        let tool = MemoryDBTool::new();

        // Set a value
        let set_args = json!({
            "operation": "set",
            "key": "name",
            "value": "Alice"
        });
        let result = tool.execute(set_args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Set 'name' = 'Alice'"));

        // Get the value
        let get_args = json!({
            "operation": "get",
            "key": "name"
        });
        let result = tool.execute(get_args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Alice"));
    }

    /// Tests deleting a value from the `MemoryDBTool`.
    #[tokio::test]
    async fn test_memory_db_delete() {
        let tool = MemoryDBTool::new();

        // Set a value
        let set_args = json!({
            "operation": "set",
            "key": "temp",
            "value": "data"
        });
        tool.execute(set_args).await.unwrap();

        // Delete the value
        let delete_args = json!({
            "operation": "delete",
            "key": "temp"
        });
        let result = tool.execute(delete_args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Deleted 'temp'"));

        // Try to get deleted value
        let get_args = json!({
            "operation": "get",
            "key": "temp"
        });
        let result = tool.execute(get_args).await.unwrap();
        assert!(!result.success);
        assert!(result.output.contains("not found"));
    }

    /// Tests checking if a key exists in the `MemoryDBTool`.
    #[tokio::test]
    async fn test_memory_db_exists() {
        let tool = MemoryDBTool::new();

        // Check non-existent key
        let exists_args = json!({
            "operation": "exists",
            "key": "test"
        });
        let result = tool.execute(exists_args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("false"));

        // Set a value
        let set_args = json!({
            "operation": "set",
            "key": "test",
            "value": "value"
        });
        tool.execute(set_args).await.unwrap();

        // Check existing key
        let exists_args = json!({
            "operation": "exists",
            "key": "test"
        });
        let result = tool.execute(exists_args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("true"));
    }

    /// Tests listing the contents of the `MemoryDBTool`.
    #[tokio::test]
    async fn test_memory_db_list() {
        let tool = MemoryDBTool::new();

        // List empty database
        let list_args = json!({
            "operation": "list"
        });
        let result = tool.execute(list_args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("empty"));

        // Add some items
        tool.execute(json!({
            "operation": "set",
            "key": "key1",
            "value": "value1"
        }))
        .await
        .unwrap();

        tool.execute(json!({
            "operation": "set",
            "key": "key2",
            "value": "value2"
        }))
        .await
        .unwrap();

        // List items
        let list_args = json!({
            "operation": "list"
        });
        let result = tool.execute(list_args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("2 items"));
        assert!(result.output.contains("key1"));
        assert!(result.output.contains("key2"));
    }

    /// Tests clearing the `MemoryDBTool`.
    #[tokio::test]
    async fn test_memory_db_clear() {
        let tool = MemoryDBTool::new();

        // Add some items
        tool.execute(json!({
            "operation": "set",
            "key": "key1",
            "value": "value1"
        }))
        .await
        .unwrap();

        tool.execute(json!({
            "operation": "set",
            "key": "key2",
            "value": "value2"
        }))
        .await
        .unwrap();

        // Clear database
        let clear_args = json!({
            "operation": "clear"
        });
        let result = tool.execute(clear_args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("2 items removed"));

        // Verify database is empty
        let list_args = json!({
            "operation": "list"
        });
        let result = tool.execute(list_args).await.unwrap();
        assert!(result.output.contains("empty"));
    }

    /// Tests an invalid operation in the `MemoryDBTool`.
    #[tokio::test]
    async fn test_memory_db_invalid_operation() {
        let tool = MemoryDBTool::new();

        let args = json!({
            "operation": "invalid_op"
        });
        let result = tool.execute(args).await;
        assert!(result.is_err());
    }

    /// Tests sharing the database between `MemoryDBTool` instances.
    #[tokio::test]
    async fn test_memory_db_shared_instance() {
        use std::sync::{Arc, Mutex};

        // Create a shared database
        let shared_db = Arc::new(Mutex::new(HashMap::new()));
        let tool1 = MemoryDBTool::with_shared_db(shared_db.clone());
        let tool2 = MemoryDBTool::with_shared_db(shared_db.clone());

        // Set value with tool1
        tool1
            .execute(json!({
                "operation": "set",
                "key": "shared",
                "value": "data"
            }))
            .await
            .unwrap();

        // Get value with tool2
        let result = tool2
            .execute(json!({
                "operation": "get",
                "key": "shared"
            }))
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.output.contains("data"));
    }

    /// Tests the WebScraperTool.
    #[tokio::test]
    async fn test_web_scraper_tool() {
        let tool = WebScraperTool;
        assert_eq!(tool.name(), "web_scraper");

        // Test with missing URL parameter
        let args = json!({});
        let result = tool.execute(args).await;
        assert!(result.is_err());

        // Note: We can't easily test the actual HTTP functionality in unit tests
        // without mocking, but we can test parameter validation
    }

    /// Tests the JsonParserTool parse operation.
    #[tokio::test]
    async fn test_json_parser_tool_parse() {
        let tool = JsonParserTool;
        assert_eq!(tool.name(), "json_parser");

        let args = json!({
            "operation": "parse",
            "json": "{\"key\": \"value\", \"number\": 42}"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("✓ JSON parsed successfully"));
        assert!(result.output.contains("Type: object"));
    }

    /// Tests the JsonParserTool stringify operation.
    #[tokio::test]
    async fn test_json_parser_tool_stringify() {
        let tool = JsonParserTool;

        let args = json!({
            "operation": "stringify",
            "json": "  {\"key\": \"value\"}  "
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("key"));
        assert!(result.output.contains("value"));
    }

    /// Tests the JsonParserTool get_value operation.
    #[tokio::test]
    async fn test_json_parser_tool_get_value() {
        let tool = JsonParserTool;

        let args = json!({
            "operation": "get_value",
            "json": "{\"user\": {\"name\": \"Alice\", \"age\": 30}}",
            "path": "user.name"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Alice"));
    }

    /// Tests the JsonParserTool validate operation.
    #[tokio::test]
    async fn test_json_parser_tool_validate() {
        let tool = JsonParserTool;

        // Valid JSON
        let args = json!({
            "operation": "validate",
            "json": "{\"valid\": true}"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("✓ JSON is valid"));

        // Invalid JSON
        let args = json!({
            "operation": "validate",
            "json": "{\"invalid\": }"
        });
        let result = tool.execute(args).await;
        assert!(result.is_ok()); // Tool returns success with error message for validation
        assert!(result.unwrap().output.contains("✗ JSON validation failed"));
    }

    /// Tests the TimestampTool now operation.
    #[tokio::test]
    async fn test_timestamp_tool_now() {
        let tool = TimestampTool;
        assert_eq!(tool.name(), "timestamp");

        let args = json!({
            "operation": "now"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Current time"));
        assert!(result.output.contains("Unix timestamp"));
        assert!(result.output.contains("RFC3339"));
    }

    /// Tests the TimestampTool format operation.
    #[tokio::test]
    async fn test_timestamp_tool_format() {
        let tool = TimestampTool;

        let args = json!({
            "operation": "format",
            "timestamp": "1640995200", // 2022-01-01 00:00:00 UTC
            "format": "%Y-%m-%d"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("2022-01-01"));
    }

    /// Tests the TimestampTool add operation.
    #[tokio::test]
    async fn test_timestamp_tool_add() {
        let tool = TimestampTool;

        let args = json!({
            "operation": "add",
            "timestamp": "2022-01-01T00:00:00Z",
            "unit": "days",
            "amount": 5
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Added 5 days"));
    }

    /// Tests the TimestampTool diff operation.
    #[tokio::test]
    async fn test_timestamp_tool_diff() {
        let tool = TimestampTool;

        let args = json!({
            "operation": "diff",
            "timestamp1": "2022-01-01T00:00:00Z",
            "timestamp2": "2022-01-02T00:00:00Z"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("86400 seconds")); // 1 day in seconds
    }

    /// Tests the FileIOTool read operation.
    #[tokio::test]
    async fn test_file_io_tool_read() {
        let tool = FileIOTool;
        assert_eq!(tool.name(), "file_io");

        // Create a temporary file for testing
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();
        std::fs::write(&file_path, "Hello, World!").unwrap();

        let args = json!({
            "operation": "read",
            "path": file_path
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Hello, World!"));
    }

    /// Tests the FileIOTool write operation.
    #[tokio::test]
    async fn test_file_io_tool_write() {
        let tool = FileIOTool;

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        let args = json!({
            "operation": "write",
            "path": file_path,
            "content": "Test content"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Wrote 12 bytes"));

        // Verify the content was written
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Test content");
    }

    /// Tests the FileIOTool exists operation.
    #[tokio::test]
    async fn test_file_io_tool_exists() {
        let tool = FileIOTool;

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        let args = json!({
            "operation": "exists",
            "path": file_path
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("exists: true"));
        assert!(result.output.contains("(file)"));
    }

    /// Tests the ShellCommandTool with a safe command.
    #[tokio::test]
    async fn test_shell_command_tool_safe() {
        let tool = ShellCommandTool;
        assert_eq!(tool.name(), "shell_command");

        // Test with a safe command
        let args = json!({
            "command": "echo 'hello world'"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("hello world"));
    }

    /// Tests the ShellCommandTool with a blocked dangerous command.
    #[tokio::test]
    async fn test_shell_command_tool_blocked() {
        let tool = ShellCommandTool;

        let args = json!({
            "command": "rm -rf /"
        });
        let result = tool.execute(args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Command blocked"));
    }

    /// Tests the HttpRequestTool with missing method.
    #[tokio::test]
    async fn test_http_request_tool_missing_method() {
        let tool = HttpRequestTool;
        assert_eq!(tool.name(), "http_request");

        let args = json!({
            "url": "https://httpbin.org/get"
        });
        let result = tool.execute(args).await;
        assert!(result.is_err());
    }

    /// Tests the FileListTool.
    #[tokio::test]
    async fn test_file_list_tool() {
        let tool = FileListTool;
        assert_eq!(tool.name(), "file_list");

        let args = json!({
            "path": ".",
            "show_hidden": false
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Directory listing"));
        assert!(result.output.contains("Total items"));
    }

    /// Tests the SystemInfoTool.
    #[tokio::test]
    async fn test_system_info_tool() {
        let tool = SystemInfoTool;
        assert_eq!(tool.name(), "system_info");

        let args = json!({
            "category": "os"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Operating System"));
        assert!(result.output.contains("OS:"));
    }

    /// Tests the TextProcessorTool search operation.
    #[tokio::test]
    async fn test_text_processor_tool_search() {
        let tool = TextProcessorTool;
        assert_eq!(tool.name(), "text_processor");

        let args = json!({
            "operation": "search",
            "text": "Hello world, hello universe",
            "pattern": "hello",
            "case_sensitive": false
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Found 2 match(es)"));
    }

    /// Tests the TextProcessorTool replace operation.
    #[tokio::test]
    async fn test_text_processor_tool_replace() {
        let tool = TextProcessorTool;

        let args = json!({
            "operation": "replace",
            "text": "Hello world",
            "pattern": "world",
            "replacement": "universe"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Replaced 1 occurrence"));
        assert!(result.output.contains("Hello universe"));
    }

    /// Tests the TextProcessorTool count operation.
    #[tokio::test]
    async fn test_text_processor_tool_count() {
        let tool = TextProcessorTool;

        let args = json!({
            "operation": "count",
            "text": "Hello\nworld"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Characters: 11"));
        assert!(result.output.contains("Lines: 2"));
        assert!(result.output.contains("Words: 2"));
    }

    /// Tests the TextProcessorTool uppercase operation.
    #[tokio::test]
    async fn test_text_processor_tool_uppercase() {
        let tool = TextProcessorTool;

        let args = json!({
            "operation": "uppercase",
            "text": "hello world"
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "HELLO WORLD");
    }

    /// Tests the TextProcessorTool trim operation.
    #[tokio::test]
    async fn test_text_processor_tool_trim() {
        let tool = TextProcessorTool;

        let args = json!({
            "operation": "trim",
            "text": "  hello world  "
        });
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "hello world");
    }
}
