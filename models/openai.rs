use async_trait::async_trait;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::model::store::STORE;
use crate::service::plugins::chatgpt::app::CHATGPT_NAMESPACE;
use crate::service::plugins::chatgpt::chat::{ChatMessage, ChatMode, ChatResponse, ChatRole, Conversation, LLMAiClient};
use crate::service::plugins::chatgpt::config::{DEFAULT_OPENAI_MODEL, get_config, OpenAIConfig};

pub struct OpenAI {
    config: OpenAIConfig,
}

impl OpenAI {
    pub fn new() -> Self {
        let config = get_config();
        Self { config: config.openai.unwrap_or_default() }
    }
}

fn system(system: &String) -> ChatMessage {
    ChatMessage {
        message: system.clone(),
        message_id: None,
        parent_id: None,
        role: ChatRole::System,
    }
}

/// reference: https://platform.openai.com/docs/api-reference/chat/create
#[derive(Serialize, Debug)]
struct OpenAIMessage {
    role: String,
    content: String,
    // tool_calls
}

impl From<ChatMessage> for OpenAIMessage {
    fn from(message: ChatMessage) -> Self {
        Self {
            role: message.role.into(),
            content: message.message.clone(),
        }
    }
}

#[derive(Serialize, Debug)]
struct OpenAIChatRequest {
    pub messages: Vec<OpenAIMessage>,
    pub model: String,
    frequency_penalty: Option<f32>,
    logit_bias: Option<f32>,
    logprobs: Option<bool>,
    top_logprobs: Option<u32>,
    max_tokens: Option<u32>,
    n: Option<u32>,
    presence_penalty: Option<f32>,
    response_format: Option<String>, // text or json_object
    seed: Option<u32>,
    stream: Option<bool>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    // tools
    // tool_choice
}

impl OpenAIChatRequest {
    pub fn new(messages: Vec<OpenAIMessage>, config: &OpenAIConfig) -> Self {
        Self {
            messages,
            model: config.model.clone().unwrap_or(DEFAULT_OPENAI_MODEL.to_string()),
            frequency_penalty: config.frequency_penalty,
            logit_bias: None,
            logprobs: None,
            top_logprobs: None,
            max_tokens: config.max_tokens,
            n: None,
            presence_penalty: config.presence_penalty,
            response_format: None,
            seed: None,
            stream: None,
            temperature: config.temperature,
            top_p: config.top_p,
        }
    }

}

/// reference https://platform.openai.com/docs/api-reference/chat/object
#[derive(Deserialize, Debug)]
struct OpenAIChatResponse {
    pub id: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIChatChoice>,
    pub object: String,
    pub usage: Option<OpenAIUsage>
}

#[derive(Deserialize, Debug)]
pub struct ChoiceMessage {
    pub content: Option<String>,
    pub role: String
}



#[derive(Deserialize, Debug)]
struct OpenAIChatChoice {
    pub message: Option<ChoiceMessage>,
    pub index: u32,
    pub logprobs: Option<OpenAILogProbs>,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct OpenAILogProbs {
    pub token: Option<String>,
    pub logprob: Option<f32>,
    pub bytes: Option<Vec<u8>>,
    pub top_logprobs: Option<Vec<OpenAITopLogProbs>>
}

#[derive(Deserialize, Debug)]
struct OpenAITopLogProbs {
    pub token: String,
    pub logprob: f32,
    pub bytes: Option<Vec<u8>>,
}

#[derive(Deserialize, Debug)]
struct OpenAIUsage {
    pub completion_tokens: Option<u32>,
    pub prompt_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}


impl Into<ChatResponse> for OpenAIChatResponse {
    fn into(self) -> ChatResponse {
        let choice = self.choices.first().unwrap();
        ChatResponse {
            message: choice.message.as_ref().map(|m| m.content.clone()).unwrap_or_default().unwrap_or_default(),
            mode: ChatMode::OpenAI,
            message_id: self.id.clone(),
            conversation_id: "".to_string(),
            parent_id: "".to_string(),
            raw: Default::default(),
        }
    }
}

#[async_trait]
impl LLMAiClient for OpenAI {
    async fn chat(&self, prompt: String, conversation_id: Option<String>, parent_id: Option<String>) -> crate::model::error::Result<ChatResponse> {
        let mut history = match conversation_id.clone() {
            None => vec![],
            Some(cid) => self.get_history(cid, parent_id.clone()).await.map(|c| c.messages).unwrap_or_default()
        };
        let config = &self.config;
        let system = config.system.as_ref().map(system);
        if let Some(system) = system {
            history.insert(0, system);
        }
        history.push(ChatMessage {
            message: prompt.clone(),
            message_id: None,
            parent_id: None,
            role: ChatRole::User,
        });
        let messages: Vec<OpenAIMessage> = history.iter().map(|m| (*m).clone().into()).collect();
        let body = OpenAIChatRequest::new(messages, config);
        info!("openai request: {:?}", body);
        let response = reqwest::Client::new()
            .post(&format!("{}/v1/chat/completions", config.base_url.as_deref().unwrap_or("https://api.openai.com")))
            .header("Authorization", format!("Bearer {}", config.api_key))
            .json(&body)
            .send().await?;
        let response = response.json::<Value>().await?;
        info!("openai response: {}", serde_json::to_string(&response).unwrap());
        let response = serde_json::from_value::<OpenAIChatResponse>(response).expect("openai response parse error");
        let mut response: ChatResponse = response.into();
        // generate uuid
        let uuid = uuid::Uuid::new_v4();
        response.conversation_id = conversation_id.unwrap_or(uuid.to_string());
        response.parent_id = parent_id.unwrap_or("".to_string());
        Ok(response)
    }

    async fn get_history(&self, conversation_id: String, parent_id: Option<String>) -> crate::model::error::Result<Conversation> {
        let parent_id = parent_id.or_else(|| {
            let ids: Option<String> = STORE.get(format!("conversation:{}:messages", conversation_id).as_str(), Some(CHATGPT_NAMESPACE.to_string()));
            if let Some(ids) = ids {
                ids.split(",").last().map(|id| id.to_string())
            } else {
                None
            }
        });
        match parent_id {
            None => {
                let conversation = Conversation {
                    conversation_id,
                    messages: vec![],
                };
                return Ok(conversation);
            }
            Some(parent) => {
                let mut messages = vec![];
                let mut parent_id = Some(parent);
                loop {
                    let message = self.get_message(parent_id.unwrap()).await?;
                    messages.push(message.clone());
                    parent_id = message.parent_id;
                    if parent_id.is_none() {
                        break;
                    }
                }
                let conversation = Conversation {
                    conversation_id,
                    messages,
                };
                Ok(conversation)
            }
        }
    }

    async fn get_message(&self, message_id: String) -> crate::model::error::Result<ChatMessage> {
        let message = STORE.get(format!("message:{}", message_id).as_str(), Some(CHATGPT_NAMESPACE.to_string())).expect("message not found");
        // message should be a json string
        let message: ChatMessage = serde_json::from_str(&message).expect("message parse error");
        Ok(message)
    }

    async fn set_message(&self, message_id: String, message: String, conversation_id: String) -> crate::model::error::Result<()> {
        STORE.set(format!("message:{}", message_id).as_str(), message, Some(CHATGPT_NAMESPACE.to_string()))?;
        let ids: Option<String> = STORE.get(format!("conversation:{}:messages", conversation_id).as_str(), Some(CHATGPT_NAMESPACE.to_string()));
        let ids = ids.unwrap_or_default();
        let ids = format!("{},{}", ids, message_id);
        STORE.set(format!("conversation:{}:messages", conversation_id).as_str(), ids, Some(CHATGPT_NAMESPACE.to_string()))?;
        Ok(())
    }
}