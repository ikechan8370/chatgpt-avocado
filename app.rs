use std::string::ToString;
use log::info;
use avocado_macro::service;
use crate::service::service::{Elements, KritorContext, Matchable, Service};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use avocado_common::Event;
use crate::model::store::{Namespace, STORE};
use crate::service::plugins::chatgpt::chat::{ChatMode, get_chat_response};
use crate::text;

#[derive(Debug, Clone, Default)]
#[service(
    name = "chatgpt",
    pattern = r"^[^!！][\s\S]*",
    events(Event::Message)
)]
struct ChatGPTPlugin;

pub const CHATGPT_NAMESPACE: &str = "chatgpt";
pub const CHATGPT_DEFAULT_MODE: &'static str = "openai";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserProgress {
    pub conversation_id: Option<String>,
    pub parent_id: Option<String>,
    pub mode: Option<ChatMode>
}

#[async_trait]
impl Service for ChatGPTPlugin {
    async fn process(&self, context: KritorContext) {
        info!("into chatgpt");
        if context.at_bot {
            let texts = context.message.clone().unwrap().elements.get_text_elements().unwrap();
            let prompt = texts.first().unwrap();
            let prompt = &prompt.text;
            info!("[chatgpt] {}", prompt);
            let sender_id = &context.message.as_ref().cloned().unwrap().sender.unwrap().uid;
            let r#use = &STORE.get("use", Some(CHATGPT_NAMESPACE.to_string())).unwrap_or(CHATGPT_DEFAULT_MODE.to_string());
            let user_progress_str = STORE.get(format!("user_progress:{}", sender_id).as_str(), Some(CHATGPT_NAMESPACE.to_string())).unwrap_or("".to_string());
            let user_progress: UserProgress = serde_json::from_str(&user_progress_str).unwrap_or_default();
            let mode = user_progress.mode.unwrap_or(ChatMode::from(r#use.as_str()));
            let response = get_chat_response(mode, prompt.clone(), user_progress.conversation_id, user_progress.parent_id).await;
            context.reply_with_quote(vec![text!(response.unwrap().message)]).await.expect("reply failed");
        }
    }
}