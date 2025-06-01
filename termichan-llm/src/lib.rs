use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequest},
    Client,
};
use thiserror::Error;
use termichan_config::LlmConfig;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("OpenAI API key not configured")]
    ApiKeyMissing,
    #[error("OpenAI API error: {0}")]
    ApiError(#[from] async_openai::error::OpenAIError),
    #[error("Empty response from OpenAI")]
    EmptyResponse,
}

/// OpenAI LLM 服务
pub struct LlmService {
    client: Client<OpenAIConfig>,
    config: LlmConfig,
}

impl LlmService {
    /// 从配置创建新的LLM服务
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        let api_key = config
            .api_key
            .as_ref()
            .ok_or(LlmError::ApiKeyMissing)?;

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1")
            .to_string();

        // 使用OpenAIConfig构建客户端
        let openai_config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(base_url);

        let client = Client::with_config(openai_config);

        Ok(Self { client, config })
    }

    /// 执行聊天补全请求
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
    ) -> Result<String, LlmError> {
        let request = CreateChatCompletionRequest {
            model: self.config.model.clone(),
            messages,
            temperature: Some(self.config.temperature),
            top_p: self.config.top_p,
            max_tokens: self.config.max_tokens.map(|v| v as u16),
            ..Default::default()
        };

        let response = self
            .client
            .chat()
            .create(request)
            .await?;

        response.choices[0]
            .message
            .content
            .clone()
            .ok_or(LlmError::EmptyResponse)
    }
}