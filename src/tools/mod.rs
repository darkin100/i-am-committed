use openai_api_rs::v1::chat_completion::{Tool, ToolType};
use openai_api_rs::v1::types::{Function, FunctionParameters, JSONSchemaDefine, JSONSchemaType};
use std::collections::HashMap;

pub mod executor;

/// Defines available tools for the LLM to use
pub fn get_tool_definitions() -> Vec<Tool> {
    vec![
        Tool {
            r#type: ToolType::Function,
            function: Function {
                name: "get_file_content".to_string(),
                description: Some("Get the full content of a specific file from the repository. Use this when you need to understand the implementation details of a file.".to_string()),
                parameters: FunctionParameters {
                    schema_type: JSONSchemaType::Object,
                    properties: Some({
                        let mut props = HashMap::new();
                        props.insert(
                            "file_path".to_string(),
                            Box::new(JSONSchemaDefine {
                                schema_type: Some(JSONSchemaType::String),
                                description: Some("The relative path to the file from the repository root".to_string()),
                                enum_values: None,
                                properties: None,
                                required: None,
                                items: None,
                            }),
                        );
                        props
                    }),
                    required: Some(vec!["file_path".to_string()]),
                },
            },
        },
        Tool {
            r#type: ToolType::Function,
            function: Function {
                name: "get_file_diff".to_string(),
                description: Some("Get the git diff for a specific file. Use this to see detailed changes for a particular file.".to_string()),
                parameters: FunctionParameters {
                    schema_type: JSONSchemaType::Object,
                    properties: Some({
                        let mut props = HashMap::new();
                        props.insert(
                            "file_path".to_string(),
                            Box::new(JSONSchemaDefine {
                                schema_type: Some(JSONSchemaType::String),
                                description: Some("The relative path to the file from the repository root".to_string()),
                                enum_values: None,
                                properties: None,
                                required: None,
                                items: None,
                            }),
                        );
                        props
                    }),
                    required: Some(vec!["file_path".to_string()]),
                },
            },
        },
        Tool {
            r#type: ToolType::Function,
            function: Function {
                name: "get_commit_history".to_string(),
                description: Some("Get recent commit messages from the repository. Use this to understand the commit message style and conventions used in this project.".to_string()),
                parameters: FunctionParameters {
                    schema_type: JSONSchemaType::Object,
                    properties: Some({
                        let mut props = HashMap::new();
                        props.insert(
                            "count".to_string(),
                            Box::new(JSONSchemaDefine {
                                schema_type: Some(JSONSchemaType::Number),
                                description: Some("Number of recent commits to retrieve (default: 5, max: 20)".to_string()),
                                enum_values: None,
                                properties: None,
                                required: None,
                                items: None,
                            }),
                        );
                        props
                    }),
                    required: None,
                },
            },
        },
        Tool {
            r#type: ToolType::Function,
            function: Function {
                name: "list_staged_files".to_string(),
                description: Some("List all files that are currently staged for commit with their status (added, modified, deleted).".to_string()),
                parameters: FunctionParameters {
                    schema_type: JSONSchemaType::Object,
                    properties: Some(HashMap::new()),
                    required: None,
                },
            },
        },
        Tool {
            r#type: ToolType::Function,
            function: Function {
                name: "get_branch_name".to_string(),
                description: Some("Get the name of the current git branch. Useful for understanding the context of the changes.".to_string()),
                parameters: FunctionParameters {
                    schema_type: JSONSchemaType::Object,
                    properties: Some(HashMap::new()),
                    required: None,
                },
            },
        },
    ]
}