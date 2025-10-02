use crate::git::GitClient;
use log::info;
use serde_json::Value;

#[derive(Debug)]
pub struct ToolExecutionError {
    pub message: String,
}

impl std::fmt::Display for ToolExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ToolExecutionError {}

pub struct ToolExecutor<'a> {
    git_client: &'a GitClient,
}

impl<'a> ToolExecutor<'a> {
    pub fn new(git_client: &'a GitClient) -> Self {
        ToolExecutor { git_client }
    }

    pub fn execute(
        &self,
        tool_name: &str,
        arguments: &Value,
    ) -> Result<String, ToolExecutionError> {
        info!(
            "Executing tool: {} with arguments: {}",
            tool_name, arguments
        );

        let result = match tool_name {
            "get_file_content" => self.get_file_content(arguments),
            "get_file_diff" => self.get_file_diff(arguments),
            "get_commit_history" => self.get_commit_history(arguments),
            "list_staged_files" => self.list_staged_files(),
            "get_branch_name" => self.get_branch_name(),
            "get_staged_changes" => self.get_staged_changes(),
            _ => Err(ToolExecutionError {
                message: format!("Unknown tool: {}", tool_name),
            }),
        };

        match &result {
            Ok(output) => info!(
                "Tool execution successful. Output length: {} chars",
                output.len()
            ),
            Err(e) => info!("Tool execution failed: {}", e),
        }

        result
    }

    fn get_file_content(&self, arguments: &Value) -> Result<String, ToolExecutionError> {
        let file_path = arguments["file_path"]
            .as_str()
            .ok_or_else(|| ToolExecutionError {
                message: "Missing or invalid 'file_path' argument".to_string(),
            })?;

        self.git_client
            .get_file_content(file_path)
            .map_err(|e| ToolExecutionError {
                message: format!("Failed to get file content: {}", e),
            })
    }

    fn get_file_diff(&self, arguments: &Value) -> Result<String, ToolExecutionError> {
        let file_path = arguments["file_path"]
            .as_str()
            .ok_or_else(|| ToolExecutionError {
                message: "Missing or invalid 'file_path' argument".to_string(),
            })?;

        self.git_client
            .get_file_diff(file_path)
            .map_err(|e| ToolExecutionError {
                message: format!("Failed to get file diff: {}", e),
            })
    }

    fn get_commit_history(&self, arguments: &Value) -> Result<String, ToolExecutionError> {
        let count = arguments["count"].as_u64().unwrap_or(5) as usize;
        let count = count.min(20); // Cap at 20

        self.git_client
            .get_commit_history(count)
            .map_err(|e| ToolExecutionError {
                message: format!("Failed to get commit history: {}", e),
            })
    }

    fn list_staged_files(&self) -> Result<String, ToolExecutionError> {
        self.git_client
            .list_staged_files_with_status()
            .map_err(|e| ToolExecutionError {
                message: format!("Failed to list staged files: {}", e),
            })
    }

    fn get_branch_name(&self) -> Result<String, ToolExecutionError> {
        self.git_client
            .get_current_branch()
            .map_err(|e| ToolExecutionError {
                message: format!("Failed to get branch name: {}", e),
            })
    }

    fn get_staged_changes(&self) -> Result<String, ToolExecutionError> {
        self.git_client
            .get_staged_changes()
            .map_err(|e| ToolExecutionError {
                message: format!("Failed to get staged changes: {}", e),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::GitClient;
    use serde_json::json;
    use std::fs::File;
    use std::io::Write;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_test_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Initialize git repo
        Command::new("git")
            .args(&["init"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();

        // Configure git user for commits
        Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();

        temp_dir
    }

    #[test]
    fn test_execute_get_branch_name() {
        let temp_dir = setup_test_repo();
        let git_client = GitClient::with_working_dir(temp_dir.path().to_string_lossy().to_string());
        let executor = ToolExecutor::new(&git_client);

        let result = executor.execute("get_branch_name", &json!({}));
        assert!(result.is_ok());
        let branch = result.unwrap();
        // Branch could be "main", "master", or the system's default
        assert!(
            !branch.is_empty(),
            "Branch name should not be empty, got: '{}'",
            branch
        );
    }

    #[test]
    fn test_execute_list_staged_files() {
        let temp_dir = setup_test_repo();
        let git_client = GitClient::with_working_dir(temp_dir.path().to_string_lossy().to_string());

        // Create and stage a test file
        let test_file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file_path).unwrap();
        writeln!(file, "test content").unwrap();

        Command::new("git")
            .args(&["add", "test.txt"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();

        let executor = ToolExecutor::new(&git_client);
        let result = executor.execute("list_staged_files", &json!({}));

        assert!(result.is_ok());
        assert!(result.unwrap().contains("test.txt"));
    }

    #[test]
    fn test_execute_unknown_tool() {
        let temp_dir = setup_test_repo();
        let git_client = GitClient::with_working_dir(temp_dir.path().to_string_lossy().to_string());
        let executor = ToolExecutor::new(&git_client);

        let result = executor.execute("unknown_tool", &json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Unknown tool"));
    }
}
