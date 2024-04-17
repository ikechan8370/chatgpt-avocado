use std::fs;
use std::io::Read;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ChatGPTConfig {
    pub openai: Option<OpenAIConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAIConfig {
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub system: Option<String>,
}


pub const OPEN_AI_BASE_URL: &str = "https://api.openai.com";
pub const DEFAULT_OPENAI_MODEL: &str = "gpt-3.5-turbo";
pub const DEFAULT_OPENAI_TEMPERATURE: f32 = 0.5;


impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            base_url: Some(OPEN_AI_BASE_URL.to_string()),
            model: Some(DEFAULT_OPENAI_MODEL.to_string()),
            api_key: String::new(),
            temperature: Some(DEFAULT_OPENAI_TEMPERATURE),
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            system: None,
        }
    }
}

impl OpenAIConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            ..Default::default()
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = api_key;
        self
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn with_frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = Some(frequency_penalty);
        self
    }

    pub fn with_presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty);
        self
    }

    pub fn with_system(mut self, system: String) -> Self {
        self.system = Some(system);
        self
    }
}

const CONFIG_PATH: &str = "src/service/plugins/chatgpt/config/config.toml";

pub fn get_config() -> ChatGPTConfig {
    let mut content = String::new();
    match fs::File::open(CONFIG_PATH) {
        Ok(mut file) => {
            file.read_to_string(&mut content).unwrap();
        }
        Err(_) => {}
    }
    let parse_result = toml::from_str(&content);
    parse_result.unwrap_or_else(|e| {
        error!("Failed to parse config: {}", e);
        ChatGPTConfig::default()
    })
}