//! # Chat Module
//!
//! This module provides the data structures for managing chat conversations.
//! It defines the roles in a conversation, the structure of a chat message,
//! and the chat session that holds the conversation history.

use serde::{Deserialize, Serialize};

/// Represents the role of a participant in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// The system, providing instructions to the assistant.
    System,
    /// The user, asking questions or giving commands.
    User,
    /// The assistant, responding to the user.
    Assistant,
    /// A tool, providing the result of a function call.
    Tool,
}

impl From<&str> for Role {
    /// Converts a string slice to a `Role`.
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "system" => Role::System,
            "user" => Role::User,
            "assistant" => Role::Assistant,
            "tool" => Role::Tool,
            _ => Role::Assistant, // Default to assistant
        }
    }
}

/// Represents a single message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The role of the message sender.
    pub role: Role,
    /// The content of the message.
    #[serde(default, deserialize_with = "deserialize_null_as_empty_string")]
    pub content: String,
    /// The name of the message sender.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Any tool calls requested by the assistant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// The ID of the tool call this message is a response to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Deserializes a null value as an empty string.
fn deserialize_null_as_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    Option::<String>::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

/// Represents a tool call requested by the assistant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// The ID of the tool call.
    pub id: String,
    /// The type of the tool call (e.g., "function").
    #[serde(rename = "type")]
    pub call_type: String,
    /// The function call to be executed.
    pub function: FunctionCall,
}

/// Represents a function call to be executed by a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// The name of the function to call.
    pub name: String,
    /// The arguments to the function, as a JSON string.
    pub arguments: String,
}

impl ChatMessage {
    /// Creates a new system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Creates a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Creates a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Creates a new tool message.
    pub fn tool(content: impl Into<String>, tool_call_id: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

/// Represents a chat session, including the conversation history and metadata.
#[derive(Debug, Clone)]
pub struct ChatSession {
    /// The messages in the chat session.
    pub messages: Vec<ChatMessage>,
    /// The system prompt for the chat session.
    pub system_prompt: Option<String>,
    /// Metadata associated with the chat session.
    pub metadata: std::collections::HashMap<String, String>,
}

impl ChatSession {
    /// Creates a new, empty chat session.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            system_prompt: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Sets the system prompt for the chat session.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Adds a message to the chat session.
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }

    /// Adds a user message to the chat session.
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.messages.push(ChatMessage::user(content));
    }

    /// Adds an assistant message to the chat session.
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.messages.push(ChatMessage::assistant(content));
    }

    /// Returns all messages in the chat session, including the system prompt.
    pub fn get_messages(&self) -> Vec<ChatMessage> {
        let mut messages = Vec::new();

        if let Some(ref system_prompt) = self.system_prompt {
            messages.push(ChatMessage::system(system_prompt.clone()));
        }

        messages.extend(self.messages.clone());
        messages
    }

    /// Clears all messages from the chat session.
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Sets a metadata key-value pair for the session.
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Gets a metadata value by key.
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Removes a metadata key-value pair.
    pub fn remove_metadata(&mut self, key: &str) -> Option<String> {
        self.metadata.remove(key)
    }

    /// Returns a summary of the chat session.
    pub fn get_summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str(&format!("Total messages: {}\n", self.messages.len()));

        let user_msgs = self
            .messages
            .iter()
            .filter(|m| matches!(m.role, Role::User))
            .count();
        let assistant_msgs = self
            .messages
            .iter()
            .filter(|m| matches!(m.role, Role::Assistant))
            .count();
        let tool_msgs = self
            .messages
            .iter()
            .filter(|m| matches!(m.role, Role::Tool))
            .count();

        summary.push_str(&format!("User messages: {}\n", user_msgs));
        summary.push_str(&format!("Assistant messages: {}\n", assistant_msgs));
        summary.push_str(&format!("Tool messages: {}\n", tool_msgs));

        if !self.metadata.is_empty() {
            summary.push_str("\nSession metadata:\n");
            for (key, value) in &self.metadata {
                summary.push_str(&format!("  {}: {}\n", key, value));
            }
        }

        summary
    }
}

impl Default for ChatSession {
    /// Creates a new, empty chat session.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the conversion from a string to a `Role`.
    #[test]
    fn test_role_from_str() {
        assert_eq!(Role::from("system"), Role::System);
        assert_eq!(Role::from("user"), Role::User);
        assert_eq!(Role::from("assistant"), Role::Assistant);
        assert_eq!(Role::from("tool"), Role::Tool);
        assert_eq!(Role::from("unknown"), Role::Assistant); // default case
        assert_eq!(Role::from("SYSTEM"), Role::System); // case insensitive
    }

    /// Tests the constructors for `ChatMessage`.
    #[test]
    fn test_chat_message_constructors() {
        let system_msg = ChatMessage::system("System message");
        assert_eq!(system_msg.role, Role::System);
        assert_eq!(system_msg.content, "System message");
        assert!(system_msg.name.is_none());
        assert!(system_msg.tool_calls.is_none());
        assert!(system_msg.tool_call_id.is_none());

        let user_msg = ChatMessage::user("User message");
        assert_eq!(user_msg.role, Role::User);
        assert_eq!(user_msg.content, "User message");

        let assistant_msg = ChatMessage::assistant("Assistant message");
        assert_eq!(assistant_msg.role, Role::Assistant);
        assert_eq!(assistant_msg.content, "Assistant message");

        let tool_msg = ChatMessage::tool("Tool result", "tool_call_123");
        assert_eq!(tool_msg.role, Role::Tool);
        assert_eq!(tool_msg.content, "Tool result");
        assert_eq!(tool_msg.tool_call_id, Some("tool_call_123".to_string()));
    }

    /// Tests the creation of a new `ChatSession`.
    #[test]
    fn test_chat_session_new() {
        let session = ChatSession::new();
        assert!(session.messages.is_empty());
        assert!(session.system_prompt.is_none());
    }

    /// Tests setting the system prompt for a `ChatSession`.
    #[test]
    fn test_chat_session_with_system_prompt() {
        let session = ChatSession::new().with_system_prompt("Test system prompt");
        assert_eq!(
            session.system_prompt,
            Some("Test system prompt".to_string())
        );
    }

    /// Tests adding a message to a `ChatSession`.
    #[test]
    fn test_chat_session_add_message() {
        let mut session = ChatSession::new();
        let msg = ChatMessage::user("Test message");
        session.add_message(msg);
        assert_eq!(session.messages.len(), 1);
    }

    /// Tests adding a user message to a `ChatSession`.
    #[test]
    fn test_chat_session_add_user_message() {
        let mut session = ChatSession::new();
        session.add_user_message("Test user message");
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].role, Role::User);
        assert_eq!(session.messages[0].content, "Test user message");
    }

    /// Tests adding an assistant message to a `ChatSession`.
    #[test]
    fn test_chat_session_add_assistant_message() {
        let mut session = ChatSession::new();
        session.add_assistant_message("Test assistant message");
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].role, Role::Assistant);
        assert_eq!(session.messages[0].content, "Test assistant message");
    }

    /// Tests getting all messages from a `ChatSession`.
    #[test]
    fn test_chat_session_get_messages() {
        let mut session = ChatSession::new().with_system_prompt("System prompt");
        session.add_user_message("User message");
        session.add_assistant_message("Assistant message");

        let messages = session.get_messages();
        assert_eq!(messages.len(), 3); // system + user + assistant
        assert_eq!(messages[0].role, Role::System);
        assert_eq!(messages[0].content, "System prompt");
        assert_eq!(messages[1].role, Role::User);
        assert_eq!(messages[1].content, "User message");
        assert_eq!(messages[2].role, Role::Assistant);
        assert_eq!(messages[2].content, "Assistant message");
    }

    /// Tests clearing all messages from a `ChatSession`.
    #[test]
    fn test_chat_session_clear() {
        let mut session = ChatSession::new();
        session.add_user_message("Test message");
        assert!(!session.messages.is_empty());

        session.clear();
        assert!(session.messages.is_empty());
    }
}
