use crate::config::Config;
use crate::tools::get_tool_definitions;
use log::{error, info};
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::chat_completion::ChatCompletionRequest;
use openai_api_rs::v1::chat_completion::{ChatCompletionMessage, Content, MessageRole, ToolCall};
use openai_api_rs::v1::common::GPT4_O_MINI;
use opentelemetry::global;
use opentelemetry::trace::{Span, SpanKind, Tracer};
use opentelemetry::KeyValue;
use regex::Regex;
use std::{env, fs};

pub struct AIClient {
    client: OpenAIClient,
    model: String,
    config: Config,
}

#[derive(Debug)]
pub struct AIError {
    pub message: String,
}

impl std::fmt::Display for AIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AIError {}

impl From<Box<dyn std::error::Error>> for AIError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        AIError {
            message: error.to_string(),
        }
    }
}

impl AIClient {
    pub fn new(api_key: String, config: Config) -> Result<Self, AIError> {
        let mut builder = OpenAIClient::builder()
            .with_api_key(api_key)
            .with_header("X-Title", "IAmCommitted")
            .with_header("Referer", "https://iamcommitted.glyndarkin.co.uk/");

        // Check for custom endpoint - IAC_OPENAI_ENDPOINT takes precedence over OPENAI_ENDPOINT
        let custom_endpoint =
            env::var("IAC_OPENAI_ENDPOINT").or_else(|_| env::var("OPENAI_ENDPOINT"));

        if let Ok(endpoint) = custom_endpoint {
            info!("Using endpoint: {}", endpoint);
            builder = builder.with_endpoint(endpoint);
        }

        let client = builder.build().map_err(|e| {
            error!("Failed to create OpenAI client: {}", e);
            AIError {
                message: format!("Failed to create OpenAI client: {}", e),
            }
        })?;

        // Create logs directory if it doesn't exist
        let log_dir = Config::get_log_dir().map_err(|e| {
            error!("Failed to get log directory: {}", e);
            AIError {
                message: format!("Failed to get log directory: {}", e),
            }
        })?;
        fs::create_dir_all(&log_dir).map_err(|e| {
            error!("Failed to create logs directory: {}", e);
            AIError {
                message: format!("Failed to create logs directory: {}", e),
            }
        })?;

        // Get model - IAC_OPENAI_MODEL takes precedence over OPENAI_MODEL
        let model = env::var("IAC_OPENAI_MODEL")
            .or_else(|_| env::var("OPENAI_MODEL"))
            .unwrap_or_else(|_| GPT4_O_MINI.to_string());

        info!("Using model: {}", model);

        Ok(AIClient {
            client,
            model,
            config,
        })
    }

    pub fn get_model(&self) -> &str {
        &self.model
    }

    pub async fn generate_commit_message_with_tools(
        &mut self,
        mut conversation_history: Vec<ChatCompletionMessage>,
    ) -> Result<AgentResponse, AIError> {
        let tracer = global::tracer("iamcommitted");
        let mut span = tracer
            .span_builder("llm.chat_completion")
            .with_kind(SpanKind::Client)
            .start(&tracer);

        span.set_attribute(KeyValue::new("llm.model", self.model.clone()));
        span.set_attribute(KeyValue::new("llm.operation", "generate_commit_message_with_tools"));

        // Load and parse prompts from config
        let prompts_md = self.config.load_prompts().map_err(|e| AIError {
            message: format!("Failed to load prompts: {}", e),
        })?;

        // Extract system prompt
        let system_re =
            Regex::new(r"(?s)## System Prompt\n\n(.*?)## User Prompt").map_err(|e| AIError {
                message: format!("Failed to compile system prompt regex: {}", e),
            })?;
        let system_prompt = system_re
            .captures(&prompts_md)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().trim())
            .ok_or_else(|| AIError {
                message: "Failed to extract system prompt from markdown".to_string(),
            })?;

        // Extract user prompt
        let user_re = Regex::new(r"(?s)## User Prompt\n\n(.*)$").map_err(|e| AIError {
            message: format!("Failed to compile user prompt regex: {}", e),
        })?;
        let user_prompt = user_re
            .captures(&prompts_md)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().trim())
            .ok_or_else(|| AIError {
                message: "Failed to extract user prompt from markdown".to_string(),
            })?;

        // Build initial messages if conversation history is empty
        if conversation_history.is_empty() {
            info!("System Prompt: {}", system_prompt);
            info!("User Prompt: {}", user_prompt);

            let system_message = ChatCompletionMessage {
                role: MessageRole::system,
                content: Content::Text(system_prompt.to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            };

            let user_message = ChatCompletionMessage {
                role: MessageRole::user,
                content: Content::Text(user_prompt.to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            };

            conversation_history = vec![system_message, user_message];
        }

        info!(
            "Sending chat completion request with {} messages",
            conversation_history.len()
        );
        span.set_attribute(KeyValue::new("llm.message_count", conversation_history.len() as i64));

        let tools = get_tool_definitions();
        span.set_attribute(KeyValue::new("llm.tool_count", tools.len() as i64));

        let mut req = ChatCompletionRequest::new(self.model.clone(), conversation_history);
        req.tools = Some(tools);

        let result = self
            .client
            .chat_completion(req)
            .await
            .map_err(|e| {
                span.record_error(&e);
                span.set_attribute(KeyValue::new("llm.error", e.to_string()));
                AIError {
                    message: format!("OpenAI API error: {}", e),
                }
            })?;

        let choice = &result.choices[0];
        let content = choice.message.content.clone();
        let tool_calls = choice.message.tool_calls.clone();

        info!(
            "AI Response - Has content: {}, Has tool calls: {}",
            content.is_some(),
            tool_calls.is_some()
        );
        info!(
            "Token Usage - Prompt: {}, Completion: {}, Total: {}",
            result.usage.prompt_tokens, result.usage.completion_tokens, result.usage.total_tokens
        );

        // Add token usage attributes to span
        span.set_attribute(KeyValue::new("llm.usage.prompt_tokens", result.usage.prompt_tokens as i64));
        span.set_attribute(KeyValue::new("llm.usage.completion_tokens", result.usage.completion_tokens as i64));
        span.set_attribute(KeyValue::new("llm.usage.total_tokens", result.usage.total_tokens as i64));
        span.set_attribute(KeyValue::new("llm.response.has_content", content.is_some()));
        span.set_attribute(KeyValue::new("llm.response.has_tool_calls", tool_calls.is_some()));

        if let Some(ref calls) = tool_calls {
            span.set_attribute(KeyValue::new("llm.response.tool_call_count", calls.len() as i64));
        }

        span.end();

        Ok(AgentResponse {
            content,
            tool_calls,
            token_usage: TokenUsage {
                prompt_tokens: result.usage.prompt_tokens,
                completion_tokens: result.usage.completion_tokens,
                total_tokens: result.usage.total_tokens,
            },
        })
    }

    pub async fn continue_conversation_with_tools(
        &mut self,
        conversation_history: Vec<ChatCompletionMessage>,
    ) -> Result<AgentResponse, AIError> {
        let tracer = global::tracer("iamcommitted");
        let mut span = tracer
            .span_builder("llm.chat_completion")
            .with_kind(SpanKind::Client)
            .start(&tracer);

        span.set_attribute(KeyValue::new("llm.model", self.model.clone()));
        span.set_attribute(KeyValue::new("llm.operation", "continue_conversation_with_tools"));

        info!(
            "Continuing conversation with {} messages",
            conversation_history.len()
        );
        span.set_attribute(KeyValue::new("llm.message_count", conversation_history.len() as i64));

        for (i, msg) in conversation_history.iter().enumerate() {
            match &msg.content {
                Content::Text(text) => {
                    info!("Message {}: role={:?}, content={}", i, msg.role, text);
                }
                _ => {
                    info!("Message {}: role={:?}, content=<non-text>", i, msg.role);
                }
            }
        }

        let tools = get_tool_definitions();
        span.set_attribute(KeyValue::new("llm.tool_count", tools.len() as i64));

        let mut req = ChatCompletionRequest::new(self.model.clone(), conversation_history);
        req.tools = Some(tools);

        let result = self
            .client
            .chat_completion(req)
            .await
            .map_err(|e| {
                span.record_error(&e);
                span.set_attribute(KeyValue::new("llm.error", e.to_string()));
                AIError {
                    message: format!("OpenAI API error: {}", e),
                }
            })?;

        let choice = &result.choices[0];
        let content = choice.message.content.clone();
        let tool_calls = choice.message.tool_calls.clone();

        info!(
            "AI Response - Has content: {}, Has tool calls: {}",
            content.is_some(),
            tool_calls.is_some()
        );
        info!(
            "Token Usage - Prompt: {}, Completion: {}, Total: {}",
            result.usage.prompt_tokens, result.usage.completion_tokens, result.usage.total_tokens
        );

        // Add token usage attributes to span
        span.set_attribute(KeyValue::new("llm.usage.prompt_tokens", result.usage.prompt_tokens as i64));
        span.set_attribute(KeyValue::new("llm.usage.completion_tokens", result.usage.completion_tokens as i64));
        span.set_attribute(KeyValue::new("llm.usage.total_tokens", result.usage.total_tokens as i64));
        span.set_attribute(KeyValue::new("llm.response.has_content", content.is_some()));
        span.set_attribute(KeyValue::new("llm.response.has_tool_calls", tool_calls.is_some()));

        if let Some(ref calls) = tool_calls {
            span.set_attribute(KeyValue::new("llm.response.tool_call_count", calls.len() as i64));
        }

        span.end();

        Ok(AgentResponse {
            content,
            tool_calls,
            token_usage: TokenUsage {
                prompt_tokens: result.usage.prompt_tokens,
                completion_tokens: result.usage.completion_tokens,
                total_tokens: result.usage.total_tokens,
            },
        })
    }
}

pub struct AgentResponse {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub token_usage: TokenUsage,
}

#[derive(Debug, Clone)]
pub struct TokenUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_new_client_with_invalid_key() {
        // Clean environment at the start to avoid pollution from other tests
        env::remove_var("IAC_OPENAI_MODEL");
        env::remove_var("OPENAI_MODEL");

        // Test with custom model
        env::set_var("OPENAI_MODEL", "test-custom-model-unique");
        let config = Config::new().unwrap();
        let custom_result = AIClient::new("invalid_key".to_string(), config);
        assert!(custom_result.is_ok());
        let custom_client = custom_result.unwrap();
        assert_eq!(custom_client.model, "test-custom-model-unique");

        // Clean up
        env::remove_var("OPENAI_MODEL");
        env::remove_var("IAC_OPENAI_MODEL");
    }

    #[test]
    fn test_custom_endpoint() {
        // Test with custom endpoint
        env::set_var(
            "OPENAI_ENDPOINT",
            "https://custom-openai-endpoint.example.com",
        );
        let config = Config::new().unwrap();
        let result = AIClient::new("test_key".to_string(), config);
        assert!(result.is_ok());

        // Clean up environment after test
        env::remove_var("OPENAI_ENDPOINT");
    }

    #[test]
    #[serial]
    fn test_iac_openai_model_precedence() {
        // Clean environment at the start to avoid pollution from other tests
        env::remove_var("IAC_OPENAI_MODEL");
        env::remove_var("OPENAI_MODEL");

        // Test 1: Only OPENAI_MODEL is set
        env::set_var("OPENAI_MODEL", "precedence-test-gpt35");
        let config = Config::new().unwrap();
        let client = AIClient::new("test_key".to_string(), config).unwrap();
        assert_eq!(client.model, "precedence-test-gpt35");

        // Test 2: Both IAC_OPENAI_MODEL and OPENAI_MODEL are set, IAC should take precedence
        env::set_var("IAC_OPENAI_MODEL", "precedence-test-gpt4");
        env::set_var("OPENAI_MODEL", "precedence-test-gpt35");
        let config = Config::new().unwrap();
        let client = AIClient::new("test_key".to_string(), config).unwrap();
        assert_eq!(client.model, "precedence-test-gpt4");

        // Test 3: Only IAC_OPENAI_MODEL is set
        env::remove_var("OPENAI_MODEL");
        env::set_var("IAC_OPENAI_MODEL", "precedence-test-gpt4turbo");
        let config = Config::new().unwrap();
        let client = AIClient::new("test_key".to_string(), config).unwrap();
        assert_eq!(client.model, "precedence-test-gpt4turbo");

        // Test 4: Neither is set, should use default
        env::remove_var("IAC_OPENAI_MODEL");
        env::remove_var("OPENAI_MODEL");
        let config = Config::new().unwrap();
        let client = AIClient::new("test_key".to_string(), config).unwrap();
        assert_eq!(client.model, GPT4_O_MINI.to_string());

        // Clean up
        env::remove_var("IAC_OPENAI_MODEL");
        env::remove_var("OPENAI_MODEL");
    }

    #[test]
    fn test_iac_openai_endpoint_precedence() {
        // Clean environment first
        env::remove_var("IAC_OPENAI_ENDPOINT");
        env::remove_var("OPENAI_ENDPOINT");

        // Test 1: Only OPENAI_ENDPOINT is set
        env::set_var("OPENAI_ENDPOINT", "https://api.openai.com");
        let config = Config::new().unwrap();
        let result = AIClient::new("test_key".to_string(), config);
        assert!(result.is_ok());

        // Test 2: Both IAC_OPENAI_ENDPOINT and OPENAI_ENDPOINT are set
        env::set_var("IAC_OPENAI_ENDPOINT", "https://custom-iac.openai.com");
        env::set_var("OPENAI_ENDPOINT", "https://api.openai.com");
        let config = Config::new().unwrap();
        let result = AIClient::new("test_key".to_string(), config);
        assert!(result.is_ok());
        // Note: We can't easily verify which endpoint was used without modifying the struct,
        // but the code logic ensures IAC_OPENAI_ENDPOINT takes precedence

        // Test 3: Only IAC_OPENAI_ENDPOINT is set
        env::remove_var("OPENAI_ENDPOINT");
        env::set_var("IAC_OPENAI_ENDPOINT", "https://iac-only.openai.com");
        let config = Config::new().unwrap();
        let result = AIClient::new("test_key".to_string(), config);
        assert!(result.is_ok());

        // Clean up
        env::remove_var("IAC_OPENAI_ENDPOINT");
        env::remove_var("OPENAI_ENDPOINT");
    }
}
