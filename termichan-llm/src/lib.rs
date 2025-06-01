use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage,
        CreateChatCompletionRequestArgs
    },
    Client,
};
use futures::StreamExt;
use thiserror::Error;
use termichan_config::LlmConfig;

/// OpenAI LLM 服务错误类型
#[derive(Error, Debug)]
pub enum LlmError {
    #[error("OpenAI API key not configured")]
    ApiKeyMissing,
    #[error("OpenAI API error: {0}")]
    ApiError(#[from] async_openai::error::OpenAIError),
    #[error("Empty response from OpenAI")]
    EmptyResponse,
}

/// 提供与OpenAI API交互的服务
///
/// 该服务封装了OpenAI的聊天补全API，支持流式和非流式响应。
/// 使用前需要通过`LlmConfig`配置API密钥和模型参数。
pub struct LlmService {
    client: Client<OpenAIConfig>,
    config: LlmConfig,
}

impl LlmService {
    /// 从配置创建新的LLM服务
    ///
    /// # 参数
    /// - `config`: LLM配置信息，必须包含有效的API密钥
    ///
    /// # 错误
    /// 如果API密钥未配置，返回`LlmError::ApiKeyMissing`
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

    /// 执行聊天补全请求（非流式）
    ///
    /// 发送消息列表并等待完整的API响应。
    ///
    /// # 参数
    /// - `messages`: 聊天消息列表，包含用户和系统的对话历史
    ///
    /// # 返回
    /// 返回完整的响应内容字符串
    ///
    /// # 错误
    /// - `LlmError::ApiError`: API请求失败
    /// - `LlmError::EmptyResponse`: API返回空响应
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
    ) -> Result<String, LlmError> {
        // 创建请求构建器并设置必要参数
        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder
            .model(&self.config.model)
            .messages(messages)
            .temperature(self.config.temperature);

        // 条件设置可选参数（使用可变引用）
        if let Some(top_p) = self.config.top_p {
            request_builder.top_p(top_p);
        }
        if let Some(max_tokens) = self.config.max_tokens {
            request_builder.max_tokens(max_tokens as u16);
        }

        let request = request_builder.build()?;

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

    /// 执行流式聊天补全请求
    ///
    /// 发送消息列表并返回响应流，适合实时显示生成内容。
    ///
    /// # 参数
    /// - `messages`: 聊天消息列表，包含用户和系统的对话历史
    ///
    /// # 返回
    /// 返回一个流，每个元素是响应内容块或错误
    ///
    /// # 错误
    /// - `LlmError::ApiError`: API请求失败
    pub async fn stream_chat_completion(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
    ) -> Result<impl futures::Stream<Item = Result<String, LlmError>>, LlmError> {
        // 创建请求构建器并设置必要参数
        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder
            .model(&self.config.model)
            .messages(messages)
            .temperature(self.config.temperature);

        // 条件设置可选参数（使用可变引用）
        if let Some(top_p) = self.config.top_p {
            request_builder.top_p(top_p);
        }
        if let Some(max_tokens) = self.config.max_tokens {
            request_builder.max_tokens(max_tokens as u16);
        }

        let request = request_builder.build()?;

        let stream = self
            .client
            .chat()
            .create_stream(request)
            .await
            .map_err(LlmError::ApiError)?;

        // 将响应流映射为字符串流
        let mapped_stream = stream.map(|chunk| {
            match chunk {
                Ok(chunk) => {
                    if let Some(choice) = chunk.choices.first() {
                        if let Some(content) = &choice.delta.content {
                            Ok(content.clone())
                        } else {
                            Err(LlmError::EmptyResponse)
                        }
                    } else {
                        Err(LlmError::EmptyResponse)
                    }
                }
                Err(e) => Err(LlmError::ApiError(e)),
            }
        });

        Ok(mapped_stream)
    }
}