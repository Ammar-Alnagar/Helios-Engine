# Chat

The Helios Engine provides a robust set of tools for managing chat conversations. At the core of this system are the `ChatMessage` and `ChatSession` structs, which allow you to create, manage, and persist conversations with agents.

## `ChatMessage`

A `ChatMessage` represents a single message in a chat conversation. It has the following properties:

- **`role`**: The role of the message sender. This can be `System`, `User`, `Assistant`, or `Tool`.
- **`content`**: The content of the message.
- **`name`**: The name of the message sender.
- **`tool_calls`**: Any tool calls requested by the assistant.
- **`tool_call_id`**: The ID of the tool call this message is a response to.

### Creating `ChatMessage`s

You can create `ChatMessage`s using the following constructor methods:

- **`ChatMessage::system(content: impl Into<String>)`**: Creates a new system message.
- **`ChatMessage::user(content: impl Into<String>)`**: Creates a new user message.
- **`ChatMessage::assistant(content: impl Into<String>)`**: Creates a new assistant message.
- **`ChatMessage::tool(content: impl Into<String>, tool_call_id: impl Into<String>)`**: Creates a new tool message.

## `ChatSession`

A `ChatSession` represents a complete chat conversation. It stores the conversation history and any associated metadata.

### Creating a `ChatSession`

You can create a new `ChatSession` using the `ChatSession::new()` method. You can also set a system prompt when you create the session:

```rust
use helios_engine::ChatSession;

let mut session = ChatSession::new()
    .with_system_prompt("You are a helpful coding assistant.");
```

### Managing Messages

The `ChatSession` provides several methods for managing messages:

- **`add_message(message: ChatMessage)`**: Adds a message to the chat session.
- **`add_user_message(content: impl Into<String>)`**: Adds a user message to the chat session.
- **`add_assistant_message(content: impl Into<String>)`**: Adds an assistant message to the chat session.
- **`get_messages()`**: Returns all messages in the chat session, including the system prompt.
- **`clear()`**: Clears all messages from the chat session.

Here's an example of how to manage a conversation with a `ChatSession`:

```rust
use helios_engine::{llm::{LLMClient, LLMProviderType}, config::LLMConfig, ChatMessage, ChatSession};

#[tokio::main]
async fn main() -> helios_engine::Result<()> {
    let llm_config = LLMConfig {
        model_name: "gpt-3.5-turbo".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        api_key: std::env::var("OPENAI_API_KEY").unwrap(),
        temperature: 0.7,
        max_tokens: 2048,
    };

    let client = LLMClient::new(LLMProviderType::Remote(llm_config)).await?;

    let mut session = ChatSession::new()
        .with_system_prompt("You are a helpful coding assistant.");

    session.add_user_message("Explain async/await in Rust");
    let response = client.chat(session.get_messages(), None, None, None, None).await?;
    session.add_assistant_message(&response.content);

    // Continue the conversation
    session.add_user_message("Can you give an example?");
    let response2 = client.chat(session.get_messages(), None, None, None, None).await?;
    session.add_assistant_message(&response2.content);

    Ok(())
}
```

### Metadata

The `ChatSession` also allows you to store and retrieve metadata associated with the conversation. This can be useful for storing information like the user's name, the current topic, or any other relevant data.

- **`set_metadata(key: impl Into<String>, value: impl Into<String>)`**: Sets a metadata key-value pair for the session.
- **`get_metadata(key: &str)`**: Gets a metadata value by key.
- **`remove_metadata(key: &str)`**: Removes a metadata key-value pair.
