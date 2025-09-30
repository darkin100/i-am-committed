use crate::ai::AIClient;
use crate::git::GitClient;
use crate::tools::executor::ToolExecutor;
use log::{info, warn};
use openai_api_rs::v1::chat_completion::{ChatCompletionMessage, Content, MessageRole, ToolCall};

#[derive(Debug)]
pub struct AgentError {
    pub message: String,
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AgentError {}

pub struct Agent<'a> {
    ai_client: &'a AIClient,
    git_client: &'a GitClient,
    max_iterations: usize,
}

impl<'a> Agent<'a> {
    pub fn new(ai_client: &'a AIClient, git_client: &'a GitClient) -> Self {
        Agent {
            ai_client,
            git_client,
            max_iterations: 10,
        }
    }

    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    pub async fn generate_commit_message(&self) -> Result<String, AgentError> {
        info!("Starting agent loop for commit message generation");

        let mut conversation_history: Vec<ChatCompletionMessage> = Vec::new();
        let tool_executor = ToolExecutor::new(self.git_client);

        // Initial request - LLM will call get_staged_changes tool
        let response = self
            .ai_client
            .generate_commit_message_with_tools(conversation_history.clone())
            .await
            .map_err(|e| AgentError {
                message: format!("Failed to generate commit message: {}", e),
            })?;

        // Add assistant's response to history
        conversation_history.push(ChatCompletionMessage {
            role: MessageRole::assistant,
            content: Content::Text(response.content.clone().unwrap_or_default()),
            name: None,
            tool_calls: response.tool_calls.clone(),
            tool_call_id: None,
        });

        // If no tool calls, return the response
        if response.tool_calls.is_none() {
            if let Some(content) = response.content {
                return Ok(content);
            }
            return Err(AgentError {
                message: "No content or tool calls in response".to_string(),
            });
        }

        // Agent loop - handle tool calls
        let mut iteration = 0;
        while iteration < self.max_iterations {
            iteration += 1;
            info!("Agent iteration {}/{}", iteration, self.max_iterations);

            // Get the tool calls from the last assistant message
            let tool_calls = match conversation_history.last() {
                Some(msg) => msg.tool_calls.clone(),
                None => None,
            };

            if tool_calls.is_none() {
                warn!("No tool calls found in conversation history");
                break;
            }

            let tool_calls = tool_calls.unwrap();

            // Execute all tool calls
            for tool_call in &tool_calls {
                let function_name = tool_call.function.name.as_deref().unwrap_or("unknown");
                let arguments_str = tool_call.function.arguments.as_deref().unwrap_or("{}");
                let arguments: serde_json::Value = serde_json::from_str(arguments_str)
                    .unwrap_or(serde_json::json!({}));

                info!("Executing tool: {}", function_name);

                let tool_result = tool_executor.execute(function_name, &arguments);

                let tool_message = ChatCompletionMessage {
                    role: MessageRole::tool,
                    content: match tool_result {
                        Ok(result) => Content::Text(result),
                        Err(e) => Content::Text(format!("Error: {}", e)),
                    },
                    name: Some(function_name.to_string()),
                    tool_calls: None,
                    tool_call_id: Some(tool_call.id.clone()),
                };

                conversation_history.push(tool_message);
            }

            // Send tool results back to LLM
            let response = self
                .ai_client
                .continue_conversation_with_tools(conversation_history.clone())
                .await
                .map_err(|e| AgentError {
                    message: format!("Failed to continue conversation: {}", e),
                })?;

            // Add assistant's response to history
            conversation_history.push(ChatCompletionMessage {
                role: MessageRole::assistant,
                content: Content::Text(response.content.clone().unwrap_or_default()),
                name: None,
                tool_calls: response.tool_calls.clone(),
                tool_call_id: None,
            });

            // If no more tool calls, we have the final answer
            if response.tool_calls.is_none() {
                if let Some(content) = response.content {
                    info!("Agent loop completed after {} iterations", iteration);
                    return Ok(content);
                }
                return Err(AgentError {
                    message: "No content in final response".to_string(),
                });
            }
        }

        warn!("Agent loop reached max iterations ({})", self.max_iterations);
        Err(AgentError {
            message: format!("Agent loop exceeded maximum iterations ({})", self.max_iterations),
        })
    }
}

pub struct AgentResponse {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}