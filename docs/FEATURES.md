# Helios Engine Features

## Overview

Helios Engine is a powerful and flexible Rust framework for building LLM-powered agents with tool support, streaming chat capabilities, and easy configuration management. Create intelligent agents that can interact with users, call tools, and maintain conversation context - with both online and offline local model support.

## Core Features

### Agent System
- **Multi-Agent Architecture**: Create multiple agents with different personalities, capabilities, and configurations
- **Agent Builder Pattern**: Easy-to-use builder for creating agents with custom system prompts and tools
- **Persistent Conversation Context**: Maintains conversation history and context for meaningful interactions
- **Tool Integration**: Agents can be equipped with various tools to extend their capabilities
- **Configurable System Prompts**: Customizable system prompts for different agent behaviors
- **Chat Session Management**: Maintains conversation state between interactions
- **LLM Provider Abstraction**: Works with multiple LLM providers through a unified interface
- **Streaming Response Support**: Real-time response streaming for interactive experiences

### Tool Support
Helios Engine comes with 18+ built-in tools for various tasks:

#### File Operations
- **File Read Tool**: Read file contents with optional line range selection (start_line, end_line)
- **File Write Tool**: Write content to files with automatic directory creation
- **File Edit Tool**: Edit files by replacing text patterns with atomic operations
- **File Search Tool**: Search for files by name patterns (with glob support) or content with recursive directory traversal
- **File List Tool**: List directory contents with detailed metadata (size, modification time, file type)
- **File I/O Tool**: Unified interface for common file operations (read, write, append, delete, copy, move, exists, size)
- **Safe File Operations**: File deletion is safe by default (only allows empty directories unless explicitly enabled recursive)

#### System & Information Tools
- **System Info Tool**: Retrieve comprehensive system information including OS, CPU, memory, disk usage, and network details
- **Timestamp Tool**: Complete timestamp operations with support for multiple formats, time arithmetic, and differences
- **JSON Parser Tool**: Full JSON operations (parse, stringify, get_value, set_value, validate) with JSONPath-like access
- **Text Processor Tool**: Advanced text operations including search, replace, split, join, count, case conversion, line/word analysis
- **Calculator Tool**: Basic arithmetic operations (+, -, *, /) with expression evaluation
- **Echo Tool**: Simple utility for echoing back messages

#### Network & Web Tools
- **Web Scraper Tool**: Fetch and extract content from web URLs with HTML text extraction
- **HTTP Request Tool**: Full HTTP method support (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS) with custom headers and body
- **Shell Command Tool**: Execute shell commands with safety restrictions (blocks dangerous operations like rm, sudo, format)

#### Data & Memory Tools
- **In-Memory Database Tool**: Key-value store with operations (set, get, delete, list, clear, exists)
- **Qdrant RAG Tool**: RAG system with vector database support (add_document, search, delete, clear operations)

#### Advanced Tools
- **Multi-Agent Communication Tools**:
  - SendMessageTool: Send messages between agents or broadcast to all
  - DelegateTaskTool: Assign tasks to other specialized agents
  - ShareContextTool: Share information in the shared context accessible to all agents

#### Improved Syntax for Adding Tools
- **New Bulk Tool Addition**: Use `.tools(vec![...])` to add multiple tools at once instead of chaining multiple `.tool()` calls
- **Cleaner Code**: More readable syntax when adding multiple tools to agents
- **Easy Organization**: Group related tools together for better code organization
- **Backward Compatible**: Old `.tool()` syntax still supported alongside new `.tools()` syntax

### ReAct (Reasoning and Acting)

Helios Engine supports the ReAct pattern, which enables agents to reason about tasks before taking actions. This leads to more thoughtful and systematic problem-solving.

**Key Features:**
- **Simple API**: Just add `.react()` to your agent builder
- **Automatic Reasoning**: Agent thinks through the problem before acting
- **Structured Planning**: Creates a clear plan before execution
- **Tool Integration**: Works seamlessly with all tools
- **Transparent Process**: Shows reasoning in output for debugging

**Basic Usage:**

```rust
use helios_engine::{Agent, Config, CalculatorTool};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let config = Config::from_file("config.toml")?;
    
    // Enable ReAct mode with a simple .react() call!
    let mut agent = Agent::builder("ReActAgent")
        .config(config)
        .system_prompt("You are a helpful assistant.")
        .tool(Box::new(CalculatorTool))
        .react()  // Enable ReAct mode
        .build()
        .await?;
    
    let response = agent.chat("Calculate (25 * 4) + (100 / 5)").await?;
    println!("{}", response);
    
    Ok(())
}
```

**How It Works:**

1. **User Query**: Agent receives the task
2. **Reasoning Phase**: Agent thinks through:
   - What is being asked?
   - What tools/information are needed?
   - What's the step-by-step plan?
3. **Action Phase**: Agent executes the plan with tools
4. **Response**: Agent provides the final answer

**When to Use ReAct:**

- Complex multi-step tasks
- Tasks requiring planning and coordination
- When you want to see the agent's thought process
- Problems that benefit from systematic approaches
- Simple, straightforward queries (adds overhead)
- When speed is more important than reasoning

**Example Output:**

```
ReAct Reasoning:
Let me think through this step by step:

1. The user wants to calculate (25 * 4) + (100 / 5)
2. I need to use the calculator tool for both operations
3. My plan:
   - First calculate 25 * 4 = 100
   - Then calculate 100 / 5 = 20
   - Finally add the results: 100 + 20 = 120

[Agent proceeds to execute the plan]
```

**Benefits:**

- **Better Accuracy**: Thinking before acting reduces errors
- **Explainability**: See how the agent approaches problems
- **Complex Tasks**: Handles multi-step problems more effectively
- **Debuggability**: Easier to identify where reasoning goes wrong

See `examples/react_agent.rs` for a complete working example!

### Multi-Agent Collaboration (Forest of Agents)
- **Agent Communication**: Agents can send messages to each other and broadcast to all agents
- **Task Delegation**: Agents can delegate tasks to other specialized agents
- **Shared Context**: All agents have access to shared context and can update it
- **Collaborative Task Execution**: Complex tasks can be distributed across multiple agents
- **Message History**: Track all communication between agents for context and debugging
- **Agent Builder Integration**: Dedicated ForestBuilder for creating and configuring multiple agents
- **Max Iterations Control**: Configurable limits for agent interaction loops to prevent infinite loops
- **Agent Roles**: Each agent can have different roles, system prompts, and capabilities for specialized tasks
- **Context Sharing**: Shared information can be accessed by all agents in the forest
- **Broadcast Messaging**: Agents can broadcast messages to all other agents
- **Team-based Processing**: Complex tasks can be broken down and distributed among specialized agents
- **Improved Syntax for Adding Multiple Agents**: Use `.agents(vec![...])` to add multiple agents at once instead of chaining multiple `.agent()` calls
- **Cleaner Code**: More readable syntax when adding multiple agents to forests
- **Easy Organization**: Group related agents together for better code organization
- **Backward Compatible**: Old `.agent()` syntax still supported alongside new `.agents()` syntax

### RAG (Retrieval-Augmented Generation) System
- **Vector Stores**: Support for both in-memory and Qdrant vector stores
- **Embedding Providers**: Integration with OpenAI embeddings and support for local models
- **Document Management**: Add, search, delete, and manage documents with metadata
- **Semantic Search**: Perform similarity searches to find relevant information
- **Document Chunking**: Automatic document chunking and preprocessing for better retrieval
- **Multiple Storage Backends**: Choose between in-memory for development and Qdrant for production
- **Cosine Similarity**: In-memory store uses cosine similarity for semantic search
- **Metadata Support**: Documents can include custom metadata for enhanced querying
- **Automatic Initialization**: Vector stores are automatically initialized when needed
- **Document IDs**: Each document is assigned a unique UUID for tracking and management
- **Batch Operations**: Support for batch adding and searching of documents

### Streaming Support
- **Real-time Response Streaming**: True real-time response streaming for both remote and local models
- **Immediate Token Delivery**: Immediate delivery of tokens as they're generated
- **Dual Mode Support**: Auto, online (remote API), and offline (local) modes
- **Stream Chunk Processing**: Handles StreamChunk responses with Delta objects
- **Progressive Response Building**: Builds complete response progressively as tokens arrive

### Local Model Support
- **llama.cpp Integration**: Run local models offline using llama.cpp with HuggingFace integration
- **Feature Flag Support**: Optional `local` feature for offline model support via cargo build flags
- **No Remote Dependency**: Full functionality available without external API calls
- **Local Config Options**: Separate LocalConfig for local model parameters (model file path, context size, etc.)
- **Hardware Acceleration**: Uses llama.cpp for optimal local inference performance
- **Offline Privacy**: Complete privacy as no data leaves the local system

### HTTP Server & API
- **OpenAI-Compatible API**: Expose OpenAI-compatible API endpoints
- **Full Parameter Support**: Support for all standard OpenAI parameters
- **Custom Endpoints**: Support for custom API endpoints and configurations
- **CORS Support**: Built-in CORS and trace support
- **Axum Framework**: Built on axum for high-performance web serving
- **API Endpoint Generation**: Automatically generates OpenAI-compatible endpoints
- **Custom Endpoint Loading**: Supports loading custom endpoint configurations from files
- **Server State Management**: Proper server state management for agents and tools
- **HTTP Route Configuration**: Flexible routing with support for various agent configurations

### Configuration Management
- **Flexible Configuration**: Easy configuration management with TOML files (config.example.toml provided)
- **Multiple Provider Support**: Works with OpenAI, Azure OpenAI, and local models
- **Environment Variables**: Support for API keys through environment variables
- **Custom Endpoints**: Support for custom endpoint configurations
- **Default Configuration**: Provides default configuration for quick setup
- **Provider Selection**: Automatic selection based on available API keys and features
- **Config Validation**: Validates configuration at runtime to ensure proper setup
- **Multiple API Key Support**: Supports different API keys for different services

## 🏗️ Architecture

### Modular Design
- **Modular Architecture**: Clean separation of concerns with dedicated modules (agent, chat, config, error, llm, tools, rag, serve, forest)
- **Extensible Tool System**: Easy to create and register custom tools using the Tool trait
- **Trait-Based Architecture**: Trait-based design for extensibility and testing
- **Re-export System**: Carefully designed re-exports for easy library usage
- **Async-First Design**: All components are designed with async-first architecture

### Error Handling
- **Custom Error Types**: Comprehensive error handling with custom `HeliosError` type
- **Result Type**: Consistent use of `Result<T, HeliosError>` type throughout the codebase
- **Detailed Error Messages**: Clear and informative error messages with specific error types (LLMError, ToolError, AgentError, etc.)
- **Error Propagation**: Consistent error propagation patterns across all modules
- **Validation Errors**: Input validation with specific error responses

### Async Support
- **Tokio Integration**: Built on Tokio for high-performance async operations
- **Async Traits**: Use of async traits for tools and other components
- **Concurrent Operations**: Support for concurrent operations and requests
- **Async/Await Syntax**: Full utilization of Rust's async/await syntax
- **Non-blocking I/O**: All I/O operations are non-blocking for optimal performance

### Type Safety
- **Rust Type System**: Leverages Rust's strong type system for safety and reliability
- **Generic Programming**: Extensive use of generics for flexible, reusable code
- **Serde Integration**: Serialization/deserialization with serde for data handling
- **Compile-time Safety**: Many errors caught at compile-time due to Rust's ownership system

## 🛠️ Development Features

### CLI Interface
- **Interactive Chat**: Start interactive chat sessions with agents using `helios-engine chat`
- **Quick Question Feature**: Ask quick questions without starting a full session using `helios-engine ask "question"`
- **Configuration Initialization**: Initialize configuration files with required API keys using `helios-engine init`
- **Command-line Options**: Full range of command-line options for different modes using clap for parsing
- **Subcommand Support**: Supports subcommands for different operations (init, chat, ask, serve)
- **Argument Validation**: Comprehensive validation of command-line arguments

### Library Integration
- **Crate Usage**: Available as a Rust crate (https://crates.io/crates/helios-engine)
- **Builder Pattern**: Consistent use of builder pattern for object creation (AgentBuilder, ForestBuilder)
- **Easy Integration**: Simple integration with existing Rust applications
- **Comprehensive API**: Full-featured API for advanced use cases
- **Documentation**: Complete documentation with examples (docs.rs support)
- **Multiple Integration Points**: Can be used both as a CLI tool and as a library

### Testing
- **Comprehensive Test Coverage**: Extensive unit and integration tests across all modules
- **Test-Driven Development**: Test-driven approach with detailed test cases for each feature
- **Mock Support**: Support for testing with mocked components
- **Async Testing**: Proper async test support with tokio::test macro
- **Integration Tests**: Real-world usage scenario testing
- **Property-Based Testing**: Testing of various input combinations and edge cases

### Documentation & Examples
- **Code Documentation**: Comprehensive rustdoc-style documentation with usage examples
- **Example Applications**: Multiple example applications in the examples/ directory
- **API Documentation**: Complete API reference documentation
- **Quick Start Guide**: Easy-to-follow quick start guide for new users
- **Usage Examples**: Multiple usage examples for different scenarios
- **Configuration Documentation**: Detailed configuration options and examples

## Usage Modes

### Online Mode
- **Remote API Support**: Connect to OpenAI, Azure OpenAI, and other compatible APIs
- **Real-time Processing**: Real-time processing with streaming responses
- **Cloud Integration**: Full integration with cloud-based LLM services
- **API Key Management**: Secure API key management from environment variables
- **Rate Limiting**: Built-in respect for API rate limits
- **Connection Management**: Efficient connection management for API calls

### Offline Mode
- **Local Model Support**: Run models locally without internet connection using llama.cpp
- **Privacy First**: Keep sensitive data on local machines with no external data transmission
- **Cost Effective**: No API costs when running local models
- **Performance Control**: Full control over local model parameters and performance
- **Hardware Utilization**: Full utilization of local hardware resources
- **Custom Model Support**: Support for various Hugging Face models locally

### Auto Mode
- **Automatic Mode Detection**: Automatically choose between online and offline based on configuration
- **Fallback Support**: Automatic fallback if primary mode fails
- **Seamless Switching**: Transparent switching between modes without application restart
- **Configuration-Based Routing**: Mode selection based on available configuration parameters

## 🔐 Security Features

### Safe Operations
- **Safe Shell Commands**: Restricted shell commands with dangerous operation blocking (rm, rmdir, format, fdisk, etc.)
- **File Operation Safety**: Safe file operations with permission checks and validation
- **Input Validation**: Comprehensive input validation and sanitization
- **Command Timeout**: Maximum timeout (60 seconds) for shell commands
- **Directory Traversal Prevention**: Protection against path traversal attacks

### Security Best Practices
- **API Key Security**: Secure handling of API keys with environment variable support
- **Parameter Sanitization**: Input sanitization for all user-provided parameters
- **Rate Limiting**: Built-in rate limiting for API calls
- **Safe Defaults**: Secure-by-default behavior (e.g., safe file deletion)
- **Input Length Limits**: Prevention of extremely large inputs that might cause issues
- **Pattern Validation**: Validation of patterns and expressions to prevent injection

## 📊 Performance Features

### Optimization
- **Memory Management**: Efficient memory usage through Rust's ownership system
- **Caching**: In-memory caching for frequently accessed data
- **Connection Pooling**: Reuse of HTTP connections for API calls
- **Async Concurrency**: Concurrent processing of multiple requests
- **Lazy Initialization**: Components are initialized only when needed
- **Streaming Processing**: Processes data streams without buffering entire content

### Scalability
- **Concurrent Agents**: Multiple agents can run concurrently
- **Shared Resources**: Efficient sharing of resources between components
- **Asynchronous Operations**: Non-blocking operations for maximum throughput
- **Resource Management**: Proper cleanup of resources to prevent leaks

## Extensibility

### Tool System
- **Custom Tool Creation**: Easy creation of new tools by implementing the Tool trait
- **Tool Registry**: Centralized management of tools with registration and retrieval
- **Tool Composition**: Tools can be combined for complex operations
- **Runtime Tool Loading**: Tools can be loaded and registered at runtime
- **Parameter Validation**: Automatic validation of tool parameters
- **Error Handling**: Consistent error handling across all tools

### Integration Points
- **LLM Provider Interface**: Easy integration of new LLM providers
- **Vector Store Interface**: Pluggable vector store implementations
- **Embedding Provider Interface**: Support for different embedding providers
- **Configuration Extensions**: Easy extension of configuration options