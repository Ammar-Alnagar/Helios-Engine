use crate::error::{HeliosError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    #[serde(skip)]
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
                description: "The directory path to search in (default: current directory)".to_string(),
                required: Some(false),
            },
        );
        params.insert(
            "pattern".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "File name pattern to search for (supports wildcards like *.rs)".to_string(),
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

        let base_path = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");
        
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
            let re_pattern = pat
                .replace(".", r"\.")
                .replace("*", ".*")
                .replace("?", ".");
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
                if file_name.starts_with('.') || 
                   file_name == "target" || 
                   file_name == "node_modules" ||
                   file_name == "__pycache__" {
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
                                results.push(format!("📄 {} (found in {} lines)", 
                                    path.display(), matching_lines.len()));
                                for (line_num, line) in matching_lines {
                                    results.push(format!("  Line {}: {}", line_num + 1, line.trim()));
                                }
                            }
                        }
                    }
                }
            }
        }

        if results.is_empty() {
            Ok(ToolResult::success("No files found matching the criteria.".to_string()))
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

        let start_line = args.get("start_line").and_then(|v| v.as_u64()).map(|n| n as usize);
        let end_line = args.get("end_line").and_then(|v| v.as_u64()).map(|n| n as usize);

        let output = if let (Some(start), Some(end)) = (start_line, end_line) {
            let lines: Vec<&str> = content.lines().collect();
            let start_idx = start.saturating_sub(1);
            let end_idx = end.min(lines.len());
            
            if start_idx >= lines.len() {
                return Err(HeliosError::ToolError(format!(
                    "Start line {} is beyond file length ({})",
                    start, lines.len()
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
            std::fs::create_dir_all(parent)
                .map_err(|e| HeliosError::ToolError(format!("Failed to create directories: {}", e)))?;
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
            return Err(HeliosError::ToolError("'find' parameter cannot be empty".to_string()));
        }

        let path = Path::new(file_path);
        let parent = path.parent().ok_or_else(|| {
            HeliosError::ToolError(format!("Invalid target path: {}", file_path))
        })?;
        let file_name = path.file_name().ok_or_else(|| {
            HeliosError::ToolError(format!("Invalid target path: {}", file_path))
        })?;

        // Build a temp file path in the same directory for atomic rename
        let pid = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| HeliosError::ToolError(format!("Clock error: {}", e)))?
            .as_nanos();
        let tmp_name = format!("{}.tmp.{}.{}", file_name.to_string_lossy(), pid, nanos);
        let tmp_path = parent.join(tmp_name);

        // Open files
        let input_file = std::fs::File::open(&path)
            .map_err(|e| HeliosError::ToolError(format!("Failed to open file for read: {}", e)))?;
        let mut reader = BufReader::new(input_file);

        let tmp_file = std::fs::File::create(&tmp_path).map_err(|e| {
            HeliosError::ToolError(format!("Failed to create temp file {}: {}", tmp_path.display(), e))
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
        writer.flush().map_err(|e| HeliosError::ToolError(format!("Failed to flush temp file: {}", e)))?;
        tmp_file.sync_all().map_err(|e| HeliosError::ToolError(format!("Failed to sync temp file: {}", e)))?;

        // Preserve permissions
        if let Ok(meta) = std::fs::metadata(&path) {
            if let Err(e) = std::fs::set_permissions(&tmp_path, meta.permissions()) {
                let _ = std::fs::remove_file(&tmp_path);
                return Err(HeliosError::ToolError(format!("Failed to set permissions: {}", e)));
            }
        }

        // Atomic replace
        std::fs::rename(&tmp_path, &path).map_err(|e| {
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

// Streamed replacement helpers
fn replace_streaming<R: Read, W: Write>(reader: &mut R, writer: &mut W, needle: &[u8], replacement: &[u8]) -> std::io::Result<usize> {
    let mut replaced = 0usize;
    let mut carry: Vec<u8> = Vec::new();
    let mut buf = [0u8; 8192];

    let tail = if needle.len() > 1 { needle.len() - 1 } else { 0 };

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

fn write_with_replacements<W: Write>(writer: &mut W, haystack: &[u8], needle: &[u8], replacement: &[u8]) -> std::io::Result<usize> {
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

#[derive(Debug, Serialize, Deserialize)]
struct QdrantPoint {
    id: String,
    vector: Vec<f32>,
    payload: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QdrantSearchRequest {
    vector: Vec<f32>,
    limit: usize,
    with_payload: bool,
    with_vector: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct QdrantSearchResponse {
    result: Vec<QdrantSearchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QdrantSearchResult {
    id: String,
    score: f64,
    payload: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

impl QdrantRAGTool {
    /// Create a new RAG tool with Qdrant backend
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

    /// Generate embeddings using OpenAI-compatible API
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = EmbeddingRequest {
            input: text.to_string(),
            model: "text-embedding-ada-002".to_string(),
        };

        let response = self
            .client
            .post(&self.embedding_api_url)
            .header("Authorization", format!("Bearer {}", self.embedding_api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Embedding API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!("Embedding failed: {}", error_text)));
        }

        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Failed to parse embedding response: {}", e)))?;

        embedding_response
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| HeliosError::ToolError("No embedding returned".to_string()))
    }

    /// Ensure collection exists in Qdrant
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
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!("Collection creation failed: {}", error_text)));
        }

        Ok(())
    }

    /// Add a document to Qdrant
    async fn add_document(&self, text: &str, metadata: HashMap<String, serde_json::Value>) -> Result<String> {
        self.ensure_collection().await?;

        // Generate embedding
        let embedding = self.generate_embedding(text).await?;

        // Create point with metadata
        let point_id = Uuid::new_v4().to_string();
        let mut payload = metadata;
        payload.insert("text".to_string(), serde_json::json!(text));
        payload.insert("timestamp".to_string(), serde_json::json!(chrono::Utc::now().to_rfc3339()));

        let point = QdrantPoint {
            id: point_id.clone(),
            vector: embedding,
            payload,
        };

        // Upload point to Qdrant
        let upsert_url = format!("{}/collections/{}/points", self.qdrant_url, self.collection_name);
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
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!("Document upload failed: {}", error_text)));
        }

        Ok(point_id)
    }

    /// Search for similar documents
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<(String, f64, String)>> {
        // Generate query embedding
        let query_embedding = self.generate_embedding(query).await?;

        // Search in Qdrant
        let search_url = format!("{}/collections/{}/points/search", self.qdrant_url, self.collection_name);
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
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!("Search request failed: {}", error_text)));
        }

        let search_response: QdrantSearchResponse = response
            .json()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Failed to parse search response: {}", e)))?;

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

    /// Delete a document by ID
    async fn delete_document(&self, doc_id: &str) -> Result<()> {
        let delete_url = format!("{}/collections/{}/points/delete", self.qdrant_url, self.collection_name);
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
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!("Delete request failed: {}", error_text)));
        }

        Ok(())
    }

    /// Clear all documents in the collection
    async fn clear_collection(&self) -> Result<()> {
        let delete_url = format!("{}/collections/{}", self.qdrant_url, self.collection_name);
        
        let response = self
            .client
            .delete(&delete_url)
            .send()
            .await
            .map_err(|e| HeliosError::ToolError(format!("Clear failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HeliosError::ToolError(format!("Clear collection failed: {}", error_text)));
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
                let text = args
                    .get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'text' for add_document".to_string()))?;

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
                let query = args
                    .get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'text' for search".to_string()))?;

                let limit = args
                    .get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5) as usize;

                let results = self.search(query, limit).await?;

                if results.is_empty() {
                    Ok(ToolResult::success("No matching documents found".to_string()))
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
                let doc_id = args
                    .get("doc_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'doc_id' for delete".to_string()))?;

                self.delete_document(doc_id).await?;
                Ok(ToolResult::success(format!("✓ Document '{}' deleted", doc_id)))
            }
            "clear" => {
                self.clear_collection().await?;
                Ok(ToolResult::success("✓ All documents cleared from collection".to_string()))
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
    pub fn new() -> Self {
        Self {
            db: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

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
                description: "Operation to perform: 'set', 'get', 'delete', 'list', 'clear', 'exists'".to_string(),
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

        let mut db = self.db.lock().map_err(|e| {
            HeliosError::ToolError(format!("Failed to lock database: {}", e))
        })?;

        match operation {
            "set" => {
                let key = args
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'key' parameter for set operation".to_string()))?;
                let value = args
                    .get("value")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'value' parameter for set operation".to_string()))?;

                db.insert(key.to_string(), value.to_string());
                Ok(ToolResult::success(format!("✓ Set '{}' = '{}'", key, value)))
            }
            "get" => {
                let key = args
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'key' parameter for get operation".to_string()))?;

                match db.get(key) {
                    Some(value) => Ok(ToolResult::success(format!("Value for '{}': {}", key, value))),
                    None => Ok(ToolResult::error(format!("Key '{}' not found", key))),
                }
            }
            "delete" => {
                let key = args
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'key' parameter for delete operation".to_string()))?;

                match db.remove(key) {
                    Some(value) => Ok(ToolResult::success(format!("✓ Deleted '{}' (was: '{}')", key, value))),
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
                Ok(ToolResult::success(format!("✓ Cleared database ({} items removed)", count)))
            }
            "exists" => {
                let key = args
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| HeliosError::ToolError("Missing 'key' parameter for exists operation".to_string()))?;

                let exists = db.contains_key(key);
                Ok(ToolResult::success(format!("Key '{}' exists: {}", key, exists)))
            }
            _ => Err(HeliosError::ToolError(format!(
                "Unknown operation '{}'. Valid operations: set, get, delete, list, clear, exists",
                operation
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success("test output");
        assert!(result.success);
        assert_eq!(result.output, "test output");
    }

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

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("test error");
        assert!(!result.success);
        assert_eq!(result.output, "test error");
    }

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

    #[tokio::test]
    async fn test_calculator_tool_multiplication() {
        let tool = CalculatorTool;
        let args = json!({"expression": "3 * 4"});
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "12");
    }

    #[tokio::test]
    async fn test_calculator_tool_division() {
        let tool = CalculatorTool;
        let args = json!({"expression": "8 / 2"});
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "4");
    }

    #[tokio::test]
    async fn test_calculator_tool_division_by_zero() {
        let tool = CalculatorTool;
        let args = json!({"expression": "8 / 0"});
        let result = tool.execute(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_calculator_tool_invalid_expression() {
        let tool = CalculatorTool;
        let args = json!({"expression": "invalid"});
        let result = tool.execute(args).await;
        assert!(result.is_err());
    }

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

    #[tokio::test]
    async fn test_echo_tool_missing_parameter() {
        let tool = EchoTool;
        let args = json!({});
        let result = tool.execute(args).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_registry_new() {
        let registry = ToolRegistry::new();
        assert!(registry.tools.is_empty());
    }

    #[tokio::test]
    async fn test_tool_registry_register_and_get() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(CalculatorTool));

        let tool = registry.get("calculator");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "calculator");
    }

    #[tokio::test]
    async fn test_tool_registry_execute() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(CalculatorTool));

        let args = json!({"expression": "5 * 6"});
        let result = registry.execute("calculator", args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "30");
    }

    #[tokio::test]
    async fn test_tool_registry_execute_nonexistent_tool() {
        let registry = ToolRegistry::new();
        let args = json!({"expression": "5 * 6"});
        let result = registry.execute("nonexistent", args).await;
        assert!(result.is_err());
    }

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
        })).await.unwrap();
        
        tool.execute(json!({
            "operation": "set",
            "key": "key2",
            "value": "value2"
        })).await.unwrap();
        
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

    #[tokio::test]
    async fn test_memory_db_clear() {
        let tool = MemoryDBTool::new();
        
        // Add some items
        tool.execute(json!({
            "operation": "set",
            "key": "key1",
            "value": "value1"
        })).await.unwrap();
        
        tool.execute(json!({
            "operation": "set",
            "key": "key2",
            "value": "value2"
        })).await.unwrap();
        
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

    #[tokio::test]
    async fn test_memory_db_invalid_operation() {
        let tool = MemoryDBTool::new();
        
        let args = json!({
            "operation": "invalid_op"
        });
        let result = tool.execute(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_memory_db_shared_instance() {
        use std::sync::{Arc, Mutex};
        
        // Create a shared database
        let shared_db = Arc::new(Mutex::new(HashMap::new()));
        let tool1 = MemoryDBTool::with_shared_db(shared_db.clone());
        let tool2 = MemoryDBTool::with_shared_db(shared_db.clone());
        
        // Set value with tool1
        tool1.execute(json!({
            "operation": "set",
            "key": "shared",
            "value": "data"
        })).await.unwrap();
        
        // Get value with tool2
        let result = tool2.execute(json!({
            "operation": "get",
            "key": "shared"
        })).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("data"));
    }
}
