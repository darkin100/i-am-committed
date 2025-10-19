use crate::ai::{AIClient, TokenUsage};
use crate::git::GitClient;
use crate::tools::executor::ToolExecutor;
use log::{info, warn};
use openai_api_rs::v1::chat_completion::{ChatCompletionMessage, Content, MessageRole};
use opentelemetry::global;
use opentelemetry::trace::{Span, SpanKind, Status, TraceContextExt, Tracer};
use opentelemetry::{Context, KeyValue};

// OpenInference semantic conventions
const OPENINFERENCE_SPAN_KIND_AGENT: &str = "AGENT";

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

pub struct CommitMessageResult {
    pub message: String,
    pub token_usage: TokenUsage,
}

pub struct Agent<'a> {
    ai_client: &'a mut AIClient,
    git_client: &'a GitClient,
    max_iterations: usize,
}

impl<'a> Agent<'a> {
    pub fn new(ai_client: &'a mut AIClient, git_client: &'a GitClient) -> Self {
        Agent {
            ai_client,
            git_client,
            max_iterations: 10,
        }
    }

    /// Sets the maximum number of iterations for the agent loop (primarily for testing)
    #[allow(dead_code)]
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    async fn build_initial_conversation_history(
        &self,
    ) -> Result<Vec<ChatCompletionMessage>, AgentError> {
        // Load prompts and build the initial conversation with system and user messages
        use crate::config::Config;
        use regex::Regex;

        let config = Config::new().map_err(|e| AgentError {
            message: format!("Failed to create config: {}", e),
        })?;

        let prompts_md = config.load_prompts().map_err(|e| AgentError {
            message: format!("Failed to load prompts: {}", e),
        })?;

        // Extract system prompt
        let system_re =
            Regex::new(r"(?s)## System Prompt\n\n(.*?)## User Prompt").map_err(|e| AgentError {
                message: format!("Failed to compile system prompt regex: {}", e),
            })?;
        let system_prompt = system_re
            .captures(&prompts_md)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().trim())
            .ok_or_else(|| AgentError {
                message: "Failed to extract system prompt from markdown".to_string(),
            })?;

        // Extract user prompt
        let user_re = Regex::new(r"(?s)## User Prompt\n\n(.*)$").map_err(|e| AgentError {
            message: format!("Failed to compile user prompt regex: {}", e),
        })?;
        let user_prompt = user_re
            .captures(&prompts_md)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().trim())
            .ok_or_else(|| AgentError {
                message: "Failed to extract user prompt from markdown".to_string(),
            })?;

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

        Ok(vec![system_message, user_message])
    }

    pub async fn generate_commit_message(&mut self) -> Result<CommitMessageResult, AgentError> {
        let tracer = global::tracer("iamcommitted");

        // Create agent span as child of current context
        let parent_cx = Context::current();
        let mut agent_span = tracer
            .span_builder("agent.generate_commit_message")
            .with_kind(SpanKind::Internal)
            .start_with_context(&tracer, &parent_cx);

        // OpenInference semantic conventions - REQUIRED attributes
        agent_span.set_attribute(KeyValue::new(
            "openinference.span.kind",
            OPENINFERENCE_SPAN_KIND_AGENT,
        ));
        agent_span.set_attribute(KeyValue::new(
            "agent.max_iterations",
            self.max_iterations as i64,
        ));
        agent_span.set_attribute(KeyValue::new("agent.name", "commit_message_generator"));

        // Set input value for agent according to OpenInference spec
        let input_description = "Generate a commit message from staged git changes";
        agent_span.set_attribute(KeyValue::new("input.value", input_description));
        agent_span.set_attribute(KeyValue::new("input.mime_type", "text/plain"));

        // Attach span to context for nested operations
        let agent_cx = parent_cx.with_span(agent_span);
        let agent_cx_clone = agent_cx.clone();
        let agent_span_ref = agent_cx_clone.span();
        let _guard = agent_cx.attach();

        info!("Starting agent loop for commit message generation");

        let tool_executor = ToolExecutor::new(self.git_client);

        // Track total token usage across all API calls
        let mut total_token_usage = TokenUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        };

        // Initial request - LLM will call get_staged_changes tool
        // Pass empty conversation history, AI client will add system/user prompts
        let response = self
            .ai_client
            .generate_commit_message_with_tools(Vec::new())
            .await
            .map_err(|e| AgentError {
                message: format!("Failed to generate commit message: {}", e),
            })?;

        // Accumulate token usage from first API call
        total_token_usage.prompt_tokens += response.token_usage.prompt_tokens;
        total_token_usage.completion_tokens += response.token_usage.completion_tokens;
        total_token_usage.total_tokens += response.token_usage.total_tokens;

        // Build conversation history from the response
        // We need to reconstruct the full conversation including system and user prompts
        let mut conversation_history =
            self.build_initial_conversation_history()
                .await
                .map_err(|e| AgentError {
                    message: format!("Failed to build conversation history: {}", e),
                })?;

        // Add assistant's response to history
        conversation_history.push(ChatCompletionMessage {
            role: MessageRole::assistant,
            content: Content::Text(response.content.clone().unwrap_or_default()),
            name: None,
            tool_calls: response.tool_calls.clone(),
            tool_call_id: None,
        });

        // If no tool calls, return the response with token usage
        if response.tool_calls.is_none() {
            if let Some(content) = response.content {
                agent_span_ref.set_attribute(KeyValue::new("agent.iterations_used", 0_i64));
                agent_span_ref.set_attribute(KeyValue::new("agent.success", true));
                agent_span_ref.set_attribute(KeyValue::new(
                    "agent.token_usage.total",
                    total_token_usage.total_tokens as i64,
                ));
                // Set output value according to OpenInference spec
                agent_span_ref.set_attribute(KeyValue::new("output.value", content.clone()));
                agent_span_ref.set_attribute(KeyValue::new("output.mime_type", "text/plain"));
                agent_span_ref.set_status(Status::Ok);
                agent_span_ref.end();
                return Ok(CommitMessageResult {
                    message: content,
                    token_usage: total_token_usage,
                });
            }
            agent_span_ref.set_attribute(KeyValue::new("agent.success", false));
            agent_span_ref.set_attribute(KeyValue::new(
                "exception.message",
                "No content or tool calls in response",
            ));
            agent_span_ref.set_status(Status::error("No content or tool calls in response"));
            agent_span_ref.end();
            return Err(AgentError {
                message: "No content or tool calls in response".to_string(),
            });
        }

        // Agent loop - handle tool calls
        let mut iteration = 0;
        while iteration < self.max_iterations {
            iteration += 1;
            info!("Agent iteration {}/{}", iteration, self.max_iterations);

            // Add event for this iteration
            agent_span_ref.add_event(
                format!("agent.iteration.{}", iteration),
                vec![KeyValue::new("iteration", iteration as i64)],
            );

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
                let arguments: serde_json::Value =
                    serde_json::from_str(arguments_str).unwrap_or(serde_json::json!({}));

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

            // Accumulate token usage from this API call
            total_token_usage.prompt_tokens += response.token_usage.prompt_tokens;
            total_token_usage.completion_tokens += response.token_usage.completion_tokens;
            total_token_usage.total_tokens += response.token_usage.total_tokens;

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
                    agent_span_ref.set_attribute(KeyValue::new("agent.iterations_used", iteration as i64));
                    agent_span_ref.set_attribute(KeyValue::new("agent.success", true));
                    agent_span_ref.set_attribute(KeyValue::new(
                        "agent.token_usage.total",
                        total_token_usage.total_tokens as i64,
                    ));
                    // Set output value according to OpenInference spec
                    agent_span_ref.set_attribute(KeyValue::new("output.value", content.clone()));
                    agent_span_ref.set_attribute(KeyValue::new("output.mime_type", "text/plain"));
                    agent_span_ref.set_status(Status::Ok);
                    agent_span_ref.end();
                    return Ok(CommitMessageResult {
                        message: content,
                        token_usage: total_token_usage,
                    });
                }
                agent_span_ref.set_attribute(KeyValue::new("agent.success", false));
                agent_span_ref.set_attribute(KeyValue::new("exception.message", "No content in final response"));
                agent_span_ref.set_status(Status::error("No content in final response"));
                agent_span_ref.end();
                return Err(AgentError {
                    message: "No content in final response".to_string(),
                });
            }
        }

        warn!(
            "Agent loop reached max iterations ({})",
            self.max_iterations
        );
        agent_span_ref.set_attribute(KeyValue::new("agent.iterations_used", iteration as i64));
        agent_span_ref.set_attribute(KeyValue::new("agent.success", false));
        agent_span_ref.set_attribute(KeyValue::new(
            "exception.message",
            format!("Agent loop exceeded maximum iterations ({})", self.max_iterations),
        ));
        agent_span_ref.set_status(Status::error("Max iterations exceeded"));
        agent_span_ref.end();
        Err(AgentError {
            message: format!(
                "Agent loop exceeded maximum iterations ({})",
                self.max_iterations
            ),
        })
    }
}
