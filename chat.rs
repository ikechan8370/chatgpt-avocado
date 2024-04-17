use async_trait::async_trait;
use reqwest::RequestBuilder;
use reqwest_eventsource::EventSource;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::err;
use crate::model::error::Result;
use crate::service::plugins::chatgpt::models::openai::OpenAI;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum ChatMode {
    #[default]
    OpenAI = 0,
    Copilot = 1,
    GEMINI = 2,
    CLAUDE = 3,
    XH = 4,
    QWEN = 5,
    GLM4 = 6,
}

impl From<&str> for ChatMode {
    fn from(s: &str) -> Self {
        match s {
            "openai" => ChatMode::OpenAI,
            "copilot" => ChatMode::Copilot,
            "gemini" => ChatMode::GEMINI,
            "claude" => ChatMode::CLAUDE,
            "xh" => ChatMode::XH,
            "qwen" => ChatMode::QWEN,
            "glm4" => ChatMode::GLM4,
            _ => ChatMode::OpenAI,
        }
    }
}

pub struct ChatResponse {
    pub message: String,
    pub mode: ChatMode,
    pub message_id: String,
    pub conversation_id: String,
    pub parent_id: String,
    pub raw: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ChatRole {
    User = 0,
    Assistant = 1,
    System = 2,
    Function = 3,
}

impl From<&str> for ChatRole {
    fn from(s: &str) -> Self {
        match s {
            "user" => ChatRole::User,
            "assistant" => ChatRole::Assistant,
            "system" => ChatRole::System,
            "function" => ChatRole::Function,
            _ => ChatRole::System,
        }
    }
}

impl From<ChatRole> for String {
    fn from(role: ChatRole) -> Self {
        match role {
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
            ChatRole::System => "system",
            ChatRole::Function => "function",
        }.to_string()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    pub message: String,
    pub message_id: Option<String>,
    pub parent_id: Option<String>,
    pub role: ChatRole,
}

pub struct Conversation {
    pub conversation_id: String,
    pub messages: Vec<ChatMessage>,
}

#[async_trait]
pub trait LLMAiClient {
    async fn chat(&self, prompt: String, conversation_id: Option<String>, parent_id: Option<String>) -> Result<ChatResponse>;
    async fn get_history(&self, conversation_id: String, parent_id: Option<String>) -> Result<Conversation>;
    async fn get_conversation(&self, conversation_id: String, parent_id: Option<String>) -> Result<Conversation> {
        self.get_history(conversation_id, parent_id).await
    }
    async fn get_message(&self, message_id: String) -> Result<ChatMessage>;
    async fn set_message(&self, message_id: String, message: String, conversation_id: String) -> Result<()>;
}

pub async fn get_chat_response(mode: ChatMode, prompt: String, conversation_id: Option<String>, message_id: Option<String>) -> Result<ChatResponse> {
    match mode {
        ChatMode::OpenAI => {
            OpenAI::new().chat(prompt, conversation_id, message_id).await
        }
        _ => err!("not implemented yet")
    }
}