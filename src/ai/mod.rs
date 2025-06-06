use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest, Content, MessageRole};
use openai_api_rs::v1::common::GPT4_O_MINI;
use std::{fs, env};
use log::{info, error};
use regex::Regex;

pub struct AIClient {
    client: OpenAIClient,
    model: String,
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
    pub fn new(api_key: String) -> Result<Self, AIError> {
        let mut builder = OpenAIClient::builder().with_api_key(api_key);
        
        // Check for custom endpoint in environment variable
        if let Ok(custom_endpoint) = env::var("OPENAI_ENDPOINT") {
            info!("Using custom OpenAI endpoint: {}", custom_endpoint);
            builder = builder.with_endpoint(custom_endpoint);
        }
        
        let client = builder.build().map_err(|e| {
            error!("Failed to create OpenAI client: {}", e);
            AIError {
                message: format!("Failed to create OpenAI client: {}", e),
            }
        })?;

        // Create logs directory if it doesn't exist
        let home = env::var("HOME").expect("Failed to get HOME directory");
        let log_dir = format!("{}/.iamcommitted/logs", home);
        fs::create_dir_all(&log_dir).map_err(|e| {
            error!("Failed to create logs directory: {}", e);
            AIError {
                message: format!("Failed to create logs directory: {}", e),
            }
        })?;

        // Get model from environment variable or use default
        let model = env::var("OPENAI_MODEL")
            .unwrap_or_else(|_| GPT4_O_MINI.to_string());
        
        info!("Using OpenAI model: {}", model);

        Ok(AIClient { client, model })
    }
    
    pub fn get_model(&self) -> &str {
        &self.model
    }

    pub async fn generate_commit_message(&self, diff: &str) -> Result<String, AIError> {
        // Load and parse prompts from markdown
        let prompts_md = fs::read_to_string("src/config/prompts.md")
            .map_err(|e| AIError {
                message: format!("Failed to read prompts markdown: {}", e),
            })?;

        // Extract system prompt
        let system_re = Regex::new(r"(?s)## System Prompt\n\n(.*?)\n\n##")
            .map_err(|e| AIError {
                message: format!("Failed to compile system prompt regex: {}", e),
            })?;
        let system_prompt = system_re.captures(&prompts_md)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().trim())
            .ok_or_else(|| AIError {
                message: "Failed to extract system prompt from markdown".to_string(),
            })?;

        // Extract user prompt
        let user_re = Regex::new(r"(?s)## User Prompt\n\n(.*)$")
            .map_err(|e| AIError {
                message: format!("Failed to compile user prompt regex: {}", e),
            })?;
        let user_prompt = user_re.captures(&prompts_md)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().trim())
            .ok_or_else(|| AIError {
                message: "Failed to extract user prompt from markdown".to_string(),
            })?;

        let system_message = chat_completion::ChatCompletionMessage {
            role: MessageRole::system,
            content: Content::Text(system_prompt.to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        };

        let user_message = chat_completion::ChatCompletionMessage {
            role: MessageRole::user,
            content: Content::Text(user_prompt.replace("{diff}", diff)),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        };

        let req = ChatCompletionRequest::new(
            self.model.clone(),
            vec![system_message, user_message],
        );

        let result = self
            .client
            .chat_completion(req)
            .await
            .map_err(|e| AIError {
                message: format!("OpenAI API error: {}", e),
            })?;

        let response = result.choices[0]
            .message
            .content
            .clone()
            .ok_or_else(|| AIError {
                message: "No content in OpenAI response".to_string(),
            })?;

        // Log the interaction
        info!("AI Request:\n{}", diff);
        info!("AI Response:\n{}", response);

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio;

    #[tokio::test]
    async fn test_generate_commit_message() {
        // This test requires a valid OpenAI API key in the environment
        if let Ok(api_key) = env::var("OPENAI_API_KEY") {
            // Set a test model or use default
            env::set_var("OPENAI_MODEL", "gpt-3.5-turbo");
            let client = AIClient::new(api_key).unwrap();
            let diff = "diff --git a/src/main.rs b/src/main.rs
                       index 123..456 789
                       --- a/src/main.rs
                       +++ b/src/main.rs
                       @@ -1,3 +1,4 @@
                       +// Add a new feature
                        fn main() {
                       -    println!(\"Hello\");
                       +    println!(\"Hello, World!\");
                        }";

            let result = client.generate_commit_message(diff).await;
            assert!(result.is_ok());
            let message = result.unwrap();
            assert!(!message.is_empty());
            // Basic format check
            assert!(message.contains(": "));
        }
    }

    #[test]
    fn test_new_client_with_invalid_key() {
        // Test with default model first
        env::remove_var("OPENAI_MODEL");
        let default_result = AIClient::new("invalid_key".to_string());
        assert!(default_result.is_ok()); // Client creation succeeds, but API calls would fail
        let default_client = default_result.unwrap();
        assert_eq!(default_client.model, GPT4_O_MINI.to_string());
        
        // Test with custom model
        env::set_var("OPENAI_MODEL", "custom-model");
        let custom_result = AIClient::new("invalid_key".to_string());
        assert!(custom_result.is_ok());
        let custom_client = custom_result.unwrap();
        assert_eq!(custom_client.model, "custom-model");
    }
    
    #[test]
    fn test_custom_endpoint() {
        // Test with custom endpoint
        env::set_var("OPENAI_ENDPOINT", "https://custom-openai-endpoint.example.com");
        let result = AIClient::new("test_key".to_string());
        assert!(result.is_ok());
        
        // Clean up environment after test
        env::remove_var("OPENAI_ENDPOINT");
    }
}
