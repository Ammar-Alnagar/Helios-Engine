# Helios Architecture

This document provides a detailed overview of the Helios framework architecture.

## Design Principles

1. **Modularity**: Each component is self-contained and can be used independently
2. **Extensibility**: Easy to add new tools, LLM providers, and functionality
3. **Type Safety**: Leverages Rust's type system for compile-time guarantees
4. **Async First**: Built on Tokio for high-performance async operations
5. **User-Friendly**: Simple API that's easy to learn and use

## Core Components

### 1. Agent (`agent.rs`)

The Agent is the central component that orchestrates all interactions.

**Responsibilities:**
- Manages conversation flow
- Coordinates between LLM and tools
- Maintains chat history
- Handles tool call loops

**Key Features:**
- Builder pattern for flexible configuration
- Automatic tool call handling
- Iteration limit to prevent infinite loops
- Thread-safe design

### 2. LLM Client (`llm.rs`)

Handles communication with LLM APIs.

**Responsibilities:**
- Format requests according to OpenAI API spec
- Send HTTP requests to LLM endpoints
- Parse and validate responses
- Handle errors gracefully

**Supported APIs:**
- OpenAI
- Azure OpenAI
- Any OpenAI-compatible API

### 3. Tool System (`tools.rs`)

Provides extensible tool functionality.

**Components:**

#### Tool Trait
Defines the interface for all tools:
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> HashMap<String, ToolParameter>;
    async fn execute(&self, args: Value) -> Result<ToolResult>;
}
```

#### ToolRegistry
Manages tool registration and execution:
- Thread-safe storage
- Name-based lookup
- Automatic schema generation
- Error handling

### 4. Chat System (`chat.rs`)

Manages conversation state and messages.

**Types:**
- `ChatMessage`: Individual message with role and content
- `ChatSession`: Conversation history container
- `Role`: Message role (System, User, Assistant, Tool)

**Features:**
- System prompt management
- Message history tracking
- Tool call support
- Serialization for API calls

### 5. Configuration (`config.rs`)

TOML-based configuration management.

**Configuration Options:**
- Model name and parameters
- API endpoint and authentication
- Temperature and token limits
- Custom settings per agent

### 6. Error Handling (`error.rs`)

Comprehensive error types using `thiserror`.

**Error Categories:**
- Configuration errors
- LLM API errors
- Tool execution errors
- Network errors
- Serialization errors

## Data Flow

### Simple Chat Flow

```
User Input
    ↓
Agent.chat()
    ↓
ChatSession.add_user_message()
    ↓
LLMClient.chat()
    ↓
[HTTP Request to LLM API]
    ↓
LLMResponse
    ↓
ChatSession.add_assistant_message()
    ↓
Return response to user
```

### Tool-Enabled Flow

```
User Input
    ↓
Agent.chat()
    ↓
ChatSession.add_user_message()
    ↓
┌─────────────────────────────────┐
│ Agent Tool Loop                  │
│                                  │
│ 1. Get messages + tool defs     │
│ 2. Send to LLM                  │
│ 3. Check for tool calls         │
│    ↓                            │
│    ├─ Yes: Execute tools        │
│    │   Add results to history   │
│    │   Loop back to step 1      │
│    │                            │
│    └─ No: Return final response │
└─────────────────────────────────┘
    ↓
Return response to user
```

## Thread Safety

Helios is designed with thread safety in mind:

- **Tool Trait**: Requires `Send + Sync`
- **LLMClient**: Uses `reqwest::Client` (thread-safe)
- **ToolRegistry**: Uses `HashMap` protected by ownership
- **Agent**: Can be wrapped in `Arc<Mutex<>>` for sharing

## Performance Considerations

### Async/Await
- All I/O operations are async
- Non-blocking HTTP requests
- Efficient task scheduling with Tokio

### Memory Management
- Chat history grows with conversation
- Call `clear_history()` for long-running sessions
- Tool results are added to context

### Rate Limiting
Users should implement their own rate limiting:
```rust
use tokio::time::{sleep, Duration};

// Simple rate limiting
sleep(Duration::from_millis(500)).await;
let response = agent.chat(message).await?;
```

## Extension Points

### 1. Custom Tools

Implement the `Tool` trait:
```rust
struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "Description" }
    fn parameters(&self) -> HashMap<String, ToolParameter> { ... }
    async fn execute(&self, args: Value) -> Result<ToolResult> { ... }
}
```

### 2. Custom LLM Providers

Implement the `LLMProvider` trait:
```rust
struct MyProvider;

#[async_trait]
impl LLMProvider for MyProvider {
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse> {
        // Custom implementation
    }
}
```

### 3. Custom Chat Backends

Extend `ChatSession` or create your own:
```rust
struct DatabaseChatSession {
    db: Database,
    session_id: String,
}

impl DatabaseChatSession {
    async fn load_history(&self) -> Result<Vec<ChatMessage>> { ... }
    async fn save_message(&self, message: ChatMessage) -> Result<()> { ... }
}
```

## Security Considerations

1. **API Keys**: Never hardcode API keys
   - Use environment variables
   - Or secure config files with restricted permissions

2. **Tool Execution**: Validate all tool inputs
   - Sanitize user-provided data
   - Implement rate limiting
   - Add execution timeouts

3. **Error Messages**: Don't leak sensitive information
   - Filter error messages sent to users
   - Log detailed errors server-side

4. **Tool Permissions**: Implement authorization
   - Check user permissions before tool execution
   - Audit tool usage

## Testing Strategy

### Unit Tests
- Test individual components in isolation
- Mock external dependencies
- Test error cases

### Integration Tests
- Test agent with real LLM (or mock server)
- Test tool execution flow
- Test multi-turn conversations

### Example Test
```rust
#[tokio::test]
async fn test_calculator_tool() {
    let tool = CalculatorTool;
    let args = json!({"expression": "2 + 2"});
    let result = tool.execute(args).await.unwrap();
    assert_eq!(result.output, "4");
}
```

## Future Enhancements

Potential improvements:

1. **Streaming Support**: Stream responses token-by-token
2. **Tool Caching**: Cache tool results for repeated calls
3. **Parallel Tools**: Execute independent tools in parallel
4. **Plugin System**: Dynamic tool loading
5. **Observability**: Built-in metrics and tracing
6. **Persistence**: Save/load conversation state
7. **Multi-Modal**: Support images, audio, etc.
8. **Agent Collaboration**: Multiple agents working together

## Comparison with Other Frameworks

### vs LangChain (Python)
- **Performance**: Faster due to Rust
- **Type Safety**: Compile-time guarantees
- **Learning Curve**: Steeper (Rust knowledge required)
- **Ecosystem**: Smaller but growing

### vs AutoGPT
- **Scope**: More focused on tool calling
- **Simplicity**: Easier to understand and extend
- **Control**: More explicit control flow

### vs Semantic Kernel (.NET)
- **Language**: Rust vs C#
- **Design**: Similar builder patterns
- **Performance**: Comparable async performance

## Resources

- [Tokio Documentation](https://tokio.rs/)
- [OpenAI API Reference](https://platform.openai.com/docs/api-reference)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
