use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest, Content, MessageRole};
use openai_api_rs::v1::common::GPT4_O_MINI;
use std::fs;
use log::{info, error};

pub struct AIClient {
    client: OpenAIClient,
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
        let client = OpenAIClient::builder()
            .with_api_key(api_key)
            .build()
            .map_err(|e| {
                error!("Failed to create OpenAI client: {}", e);
                AIError {
                    message: format!("Failed to create OpenAI client: {}", e),
                }
            })?;

        // Create logs directory if it doesn't exist
        fs::create_dir_all("logs").map_err(|e| {
            error!("Failed to create logs directory: {}", e);
            AIError {
                message: format!("Failed to create logs directory: {}", e),
            }
        })?;

        Ok(AIClient { client })
    }

    pub async fn generate_commit_message(&self, diff: &str) -> Result<String, AIError> {
        let system_message = chat_completion::ChatCompletionMessage {
            role: MessageRole::system,
            content: Content::Text(String::from(
                "Generate a commit message following the Conventional Commits specification. \
                Use one of these types: feat, fix, chore, docs, style, refactor, perf, test, build, ci, revert. \
                Include a scope in parentheses if relevant. \
                Example format: \
                type(scope): description\n\n[optional body]"
            )),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        };

        let user_message = chat_completion::ChatCompletionMessage {
            role: MessageRole::user,
            content: Content::Text(diff.to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        };

        let req = ChatCompletionRequest::new(
            GPT4_O_MINI.to_string(),
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
        let result = AIClient::new("invalid_key".to_string());
        assert!(result.is_ok()); // Client creation succeeds, but API calls would fail
    }
}
