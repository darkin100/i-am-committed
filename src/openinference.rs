/// OpenInference semantic conventions helper functions
///
/// This module provides utilities for properly formatting span attributes
/// according to the OpenInference tracing specification.
use openai_api_rs::v1::chat_completion::{
    ChatCompletionMessage, ChatCompletionMessageForResponse, Content, Tool,
};
use opentelemetry::trace::Span;
use opentelemetry::KeyValue;

/// Flatten and set input messages on a span according to OpenInference conventions
///
/// Input messages are flattened using the pattern:
/// - `llm.input_messages.<index>.message.role`
/// - `llm.input_messages.<index>.message.content`
pub fn set_input_messages<S: Span>(span: &mut S, messages: &[ChatCompletionMessage]) {
    for (i, msg) in messages.iter().enumerate() {
        // Set role
        span.set_attribute(KeyValue::new(
            format!("llm.input_messages.{}.message.role", i),
            format!("{:?}", msg.role).to_lowercase(),
        ));

        // Set content if present
        if let Content::Text(text) = &msg.content {
            span.set_attribute(KeyValue::new(
                format!("llm.input_messages.{}.message.content", i),
                text.clone(),
            ));
        }

        // Set tool_call_id if present (for tool result messages)
        if let Some(tool_call_id) = &msg.tool_call_id {
            span.set_attribute(KeyValue::new(
                format!("llm.input_messages.{}.message.tool_call_id", i),
                tool_call_id.clone(),
            ));
        }

        // Set name if present (for tool messages)
        if let Some(name) = &msg.name {
            span.set_attribute(KeyValue::new(
                format!("llm.input_messages.{}.message.name", i),
                name.clone(),
            ));
        }

        // Set tool_calls if present (assistant messages can have tool calls)
        if let Some(tool_calls) = &msg.tool_calls {
            for (tc_idx, tool_call) in tool_calls.iter().enumerate() {
                span.set_attribute(KeyValue::new(
                    format!("llm.input_messages.{}.message.tool_calls.{}.tool_call.id", i, tc_idx),
                    tool_call.id.clone(),
                ));

                if let Some(function_name) = &tool_call.function.name {
                    span.set_attribute(KeyValue::new(
                        format!("llm.input_messages.{}.message.tool_calls.{}.tool_call.function.name", i, tc_idx),
                        function_name.clone(),
                    ));
                }

                if let Some(arguments) = &tool_call.function.arguments {
                    span.set_attribute(KeyValue::new(
                        format!("llm.input_messages.{}.message.tool_calls.{}.tool_call.function.arguments", i, tc_idx),
                        arguments.clone(),
                    ));
                }
            }
        }
    }
}

/// Flatten and set output messages on a span according to OpenInference conventions
///
/// Output messages are flattened using the pattern:
/// - `llm.output_messages.<index>.message.role`
/// - `llm.output_messages.<index>.message.content`
/// - `llm.output_messages.<index>.message.tool_calls.<toolCallIndex>.tool_call.*`
pub fn set_output_message<S: Span>(span: &mut S, message: &ChatCompletionMessageForResponse) {
    // Set role
    span.set_attribute(KeyValue::new(
        "llm.output_messages.0.message.role",
        format!("{:?}", message.role).to_lowercase(),
    ));

    // Set content if present
    if let Some(ref content) = message.content {
        span.set_attribute(KeyValue::new(
            "llm.output_messages.0.message.content",
            content.clone(),
        ));
    }

    // Set tool_calls if present
    if let Some(tool_calls) = &message.tool_calls {
        for (i, tool_call) in tool_calls.iter().enumerate() {
            span.set_attribute(KeyValue::new(
                format!("llm.output_messages.0.message.tool_calls.{}.tool_call.id", i),
                tool_call.id.clone(),
            ));

            if let Some(function_name) = &tool_call.function.name {
                span.set_attribute(KeyValue::new(
                    format!("llm.output_messages.0.message.tool_calls.{}.tool_call.function.name", i),
                    function_name.clone(),
                ));
            }

            if let Some(arguments) = &tool_call.function.arguments {
                span.set_attribute(KeyValue::new(
                    format!("llm.output_messages.0.message.tool_calls.{}.tool_call.function.arguments", i),
                    arguments.clone(),
                ));
            }
        }
    }
}

/// Flatten and set tool definitions on a span according to OpenInference conventions
///
/// Tools are flattened using the pattern:
/// - `llm.tools.<index>.tool.json_schema`
pub fn set_tools<S: Span>(span: &mut S, tools: &[Tool]) {
    for (i, tool) in tools.iter().enumerate() {
        // Serialize the entire tool as JSON schema
        if let Ok(json_schema) = serde_json::to_string(tool) {
            span.set_attribute(KeyValue::new(
                format!("llm.tools.{}.tool.json_schema", i),
                json_schema,
            ));
        }
    }
}
