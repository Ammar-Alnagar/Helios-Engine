use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

impl From<&str> for Role {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    #[serde(default, deserialize_with = "deserialize_null_as_empty_string")]
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

fn deserialize_null_as_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    Option::<String>::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

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

#[derive(Debug, Clone)]
pub struct ChatSession {
    pub messages: Vec<ChatMessage>,
    pub system_prompt: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl ChatSession {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            system_prompt: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }

    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.messages.push(ChatMessage::user(content));
    }

    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.messages.push(ChatMessage::assistant(content));
    }

    pub fn get_messages(&self) -> Vec<ChatMessage> {
        let mut messages = Vec::new();

        if let Some(ref system_prompt) = self.system_prompt {
            messages.push(ChatMessage::system(system_prompt.clone()));
        }

        messages.extend(self.messages.clone());
        messages
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }
    
    // Session memory methods
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
    
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    pub fn remove_metadata(&mut self, key: &str) -> Option<String> {
        self.metadata.remove(key)
    }
    
    pub fn get_summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str(&format!("Total messages: {}\n", self.messages.len()));
        
        let user_msgs = self.messages.iter().filter(|m| matches!(m.role, Role::User)).count();
        let assistant_msgs = self.messages.iter().filter(|m| matches!(m.role, Role::Assistant)).count();
        let tool_msgs = self.messages.iter().filter(|m| matches!(m.role, Role::Tool)).count();
        
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
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_from_str() {
        assert_eq!(Role::from("system"), Role::System);
        assert_eq!(Role::from("user"), Role::User);
        assert_eq!(Role::from("assistant"), Role::Assistant);
        assert_eq!(Role::from("tool"), Role::Tool);
        assert_eq!(Role::from("unknown"), Role::Assistant); // default case
        assert_eq!(Role::from("SYSTEM"), Role::System); // case insensitive
    }

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

    #[test]
    fn test_chat_session_new() {
        let session = ChatSession::new();
        assert!(session.messages.is_empty());
        assert!(session.system_prompt.is_none());
    }

    #[test]
    fn test_chat_session_with_system_prompt() {
        let session = ChatSession::new().with_system_prompt("Test system prompt");
        assert_eq!(
            session.system_prompt,
            Some("Test system prompt".to_string())
        );
    }

    #[test]
    fn test_chat_session_add_message() {
        let mut session = ChatSession::new();
        let msg = ChatMessage::user("Test message");
        session.add_message(msg);
        assert_eq!(session.messages.len(), 1);
    }

    #[test]
    fn test_chat_session_add_user_message() {
        let mut session = ChatSession::new();
        session.add_user_message("Test user message");
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].role, Role::User);
        assert_eq!(session.messages[0].content, "Test user message");
    }

    #[test]
    fn test_chat_session_add_assistant_message() {
        let mut session = ChatSession::new();
        session.add_assistant_message("Test assistant message");
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].role, Role::Assistant);
        assert_eq!(session.messages[0].content, "Test assistant message");
    }

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

    #[test]
    fn test_chat_session_clear() {
        let mut session = ChatSession::new();
        session.add_user_message("Test message");
        assert!(!session.messages.is_empty());

        session.clear();
        assert!(session.messages.is_empty());
    }
}
