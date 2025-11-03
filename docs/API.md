# Helios API Reference

Complete API documentation for the Helios framework.

## Core Modules

### `helios::agent`

Agent system for creating and managing LLM agents.

#### `Agent`

The main agent struct.

```rust
pub struct Agent {
    // Private fields
}
```

**Methods:**

##### `Agent::builder`
```rust
pub fn builder(name: impl Into<String>) -> AgentBuilder
```
Create an agent builder for flexible configuration.

##### `Agent::chat`
```rust
pub async fn chat(&mut self, message: impl Into<String>) -> Result<String>
```
Send a message to the agent and receive a response.

**Example:**
```rust
let response = agent.chat("Hello!").await?;
```

##### `Agent::register_tool`
```rust
pub fn register_tool(&mut self, tool: Box<dyn Tool>)
```
Register a tool with the agent.

##### `Agent::clear_history`
```rust
pub fn clear_history(&mut self)
```
Clear the conversation history.

##### `Agent::set_system_prompt`
```rust
pub fn set_system_prompt(&mut self, prompt: impl Into<String>)
```
Set or update the system prompt.

##### `Agent::set_max_iterations`
```rust
pub fn set_max_iterations(&mut self, max: usize)
```
Set the maximum number of tool call iterations.

#### `AgentBuilder`

Builder for creating agents.

```rust
pub struct AgentBuilder {
    // Private fields
}
```

**Methods:**

##### `AgentBuilder::new`
```rust
pub fn new(name: impl Into<String>) -> Self
```

##### `AgentBuilder::config`
```rust
pub fn config(self, config: Config) -> Self
```
Set the configuration.

##### `AgentBuilder::system_prompt`
```rust
pub fn system_prompt(self, prompt: impl Into<String>) -> Self
```
Set the system prompt.

##### `AgentBuilder::tool`
```rust
pub fn tool(self, tool: Box<dyn Tool>) -> Self
```
Add a tool to the agent.

##### `AgentBuilder::max_iterations`
```rust
pub fn max_iterations(self, max: usize) -> Self
```
Set maximum tool call iterations.

##### `AgentBuilder::build`
```rust
pub fn build(self) -> Result<Agent>
```
Build the agent.

**Example:**
```rust
let agent = Agent::builder("MyAgent")
    .config(config)
    .system_prompt("You are helpful")
    .tool(Box::new(CalculatorTool))
    .max_iterations(5)
    .build()
    .await?;
```

---

### `helios::config`

Configuration management.

#### `Config`

Main configuration struct.

```rust
pub struct Config {
    pub llm: LLMConfig,
}
```

**Methods:**

##### `Config::from_file`
```rust
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>
```
Load configuration from a TOML file.

##### `Config::default`
```rust
pub fn default() -> Self
```
Create a default configuration.

##### `Config::save`
```rust
pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()>
```
Save configuration to a file.

#### `LLMConfig`

LLM-specific configuration.

```rust
pub struct LLMConfig {
    pub model_name: String,
    pub base_url: String,
    pub api_key: String,
    pub temperature: f32,
    pub max_tokens: u32,
}
```

---

### `helios::llm`

LLM client and provider traits.

#### `LLMClient`

HTTP client for LLM APIs.

```rust
pub struct LLMClient {
    // Private fields
}
```

**Methods:**

##### `LLMClient::new`
```rust
pub fn new(config: LLMConfig) -> Self
```

##### `LLMClient::chat`
```rust
pub async fn chat(
    &self,
    messages: Vec<ChatMessage>,
    tools: Option<Vec<ToolDefinition>>,
) -> Result<ChatMessage>
```

#### `LLMProvider`

Trait for LLM providers.

```rust
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse>;
}
```

---

### `helios::tools`

Tool system and registry.

#### `Tool`

Trait for implementing tools.

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> HashMap<String, ToolParameter>;
    async fn execute(&self, args: Value) -> Result<ToolResult>;
}
```

**Example Implementation:**
```rust
use async_trait::async_trait;
use helios_engine::{Tool, ToolParameter, ToolResult};

struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str {
        "my_tool"
    }
    
    fn description(&self) -> &str {
        "Does something useful"
    }
    
    fn parameters(&self) -> HashMap<String, ToolParameter> {
        let mut params = HashMap::new();
        params.insert(
            "input".to_string(),
            ToolParameter {
                param_type: "string".to_string(),
                description: "Input parameter".to_string(),
                required: Some(true),
            },
        );
        params
    }
    
    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let input = args["input"].as_str().unwrap_or("");
        Ok(ToolResult::success(format!("Processed: {}", input)))
    }
}
```

#### `ToolRegistry`

Registry for managing tools.

```rust
pub struct ToolRegistry {
    // Private fields
}
```

**Methods:**

##### `ToolRegistry::new`
```rust
pub fn new() -> Self
```

##### `ToolRegistry::register`
```rust
pub fn register(&mut self, tool: Box<dyn Tool>)
```

##### `ToolRegistry::execute`
```rust
pub async fn execute(&self, name: &str, args: Value) -> Result<ToolResult>
```

##### `ToolRegistry::get_definitions`
```rust
pub fn get_definitions(&self) -> Vec<ToolDefinition>
```

##### `ToolRegistry::list_tools`
```rust
pub fn list_tools(&self) -> Vec<String>
```

#### `ToolResult`

Result of tool execution.

```rust
pub struct ToolResult {
    pub success: bool,
    pub output: String,
}
```

**Methods:**

##### `ToolResult::success`
```rust
pub fn success(output: impl Into<String>) -> Self
```

##### `ToolResult::error`
```rust
pub fn error(message: impl Into<String>) -> Self
```

#### `ToolParameter`

Tool parameter definition.

```rust
pub struct ToolParameter {
    pub param_type: String,
    pub description: String,
    pub required: Option<bool>,
}
```

---

### `helios::chat`

Chat message and session types.

#### `ChatMessage`

A single chat message.

```rust
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub name: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}
```

**Methods:**

##### `ChatMessage::system`
```rust
pub fn system(content: impl Into<String>) -> Self
```

##### `ChatMessage::user`
```rust
pub fn user(content: impl Into<String>) -> Self
```

##### `ChatMessage::assistant`
```rust
pub fn assistant(content: impl Into<String>) -> Self
```

##### `ChatMessage::tool`
```rust
pub fn tool(content: impl Into<String>, tool_call_id: impl Into<String>) -> Self
```

#### `Role`

Message role enum.

```rust
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}
```

#### `ChatSession`

Manages conversation history.

```rust
pub struct ChatSession {
    pub messages: Vec<ChatMessage>,
    pub system_prompt: Option<String>,
}
```

**Methods:**

##### `ChatSession::new`
```rust
pub fn new() -> Self
```

##### `ChatSession::with_system_prompt`
```rust
pub fn with_system_prompt(self, prompt: impl Into<String>) -> Self
```

##### `ChatSession::add_message`
```rust
pub fn add_message(&mut self, message: ChatMessage)
```

##### `ChatSession::add_user_message`
```rust
pub fn add_user_message(&mut self, content: impl Into<String>)
```

##### `ChatSession::add_assistant_message`
```rust
pub fn add_assistant_message(&mut self, content: impl Into<String>)
```

##### `ChatSession::get_messages`
```rust
pub fn get_messages(&self) -> Vec<ChatMessage>
```

##### `ChatSession::clear`
```rust
pub fn clear(&mut self)
```

---

### `helios::error`

Error types and Result alias.

#### `HeliosError`

Main error enum.

```rust
pub enum HeliosError {
    ConfigError(String),
    LLMError(String),
    ToolError(String),
    AgentError(String),
    NetworkError(reqwest::Error),
    SerializationError(serde_json::Error),
    IoError(std::io::Error),
    TomlError(toml::de::Error),
}
```

#### `Result<T>`

Type alias for `std::result::Result<T, HeliosError>`.

```rust
pub type Result<T> = std::result::Result<T, HeliosError>;
```

---

## Built-in Tools

### `CalculatorTool`

Performs basic arithmetic operations.

```rust
pub struct CalculatorTool;
```

**Parameters:**
- `expression` (string, required): Mathematical expression

**Example:**
```rust
use helios_engine::CalculatorTool;

let mut agent = Agent::builder("MathBot")
    .config(config)
    .tool(Box::new(CalculatorTool))
    .build()
    .await?;
```

### `EchoTool`

Echoes back a message.

```rust
pub struct EchoTool;
```

**Parameters:**
- `message` (string, required): Message to echo

**Example:**
```rust
use helios_engine::EchoTool;

let mut agent = Agent::builder("EchoBot")
    .config(config)
    .tool(Box::new(EchoTool))
    .build()
    .await?;
```

---

## Usage Examples

### Basic Agent
```rust
use helios_engine::{Agent, Config};

let config = Config::from_file("config.toml")?;
let mut agent = Agent::builder("Assistant")
    .config(config)
    .system_prompt("You are helpful")
    .build()
    .await?;

let response = agent.chat("Hello").await?;
```

### Agent with Tools
```rust
use helios_engine::{Agent, Config, CalculatorTool};

let config = Config::from_file("config.toml")?;
let mut agent = Agent::builder("MathBot")
    .config(config)
    .tool(Box::new(CalculatorTool))
    .build()
    .await?;

let response = agent.chat("What is 10 * 5?").await?;
```

### Custom Tool
```rust
use async_trait::async_trait;
use helios_engine::{Tool, ToolParameter, ToolResult};

struct CustomTool;

#[async_trait]
impl Tool for CustomTool {
    fn name(&self) -> &str { "custom" }
    fn description(&self) -> &str { "Custom tool" }
    fn parameters(&self) -> HashMap<String, ToolParameter> {
        HashMap::new()
    }
    async fn execute(&self, _args: Value) -> helios::Result<ToolResult> {
        Ok(ToolResult::success("Done!"))
    }
}
```

---

## Type Hierarchy

```
helios
├── Agent
│   ├── LLMClient
│   ├── ToolRegistry
│   └── ChatSession
├── Config
│   └── LLMConfig
├── Tool (trait)
│   ├── CalculatorTool
│   ├── EchoTool
│   └── [Your Custom Tools]
└── Error
    └── HeliosError
```

---

For more examples, see the [examples](../examples/) directory.
