use std::process::Command;
use std::process::Output;

pub struct GitClient {
    working_dir: Option<String>,
}

#[derive(Debug)]
pub struct GitError {
    pub message: String,
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for GitError {}

impl GitClient {
    pub fn new() -> Self {
        GitClient { working_dir: None }
    }

    pub fn with_working_dir(working_dir: String) -> Self {
        GitClient {
            working_dir: Some(working_dir),
        }
    }

    pub fn get_staged_changes(&self) -> Result<String, GitError> {
        let output = self.run_git_command(&["diff", "--cached", "--diff-algorithm=minimal"])?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn get_staged_files(&self) -> Result<String, GitError> {
        let output = self.run_git_command(&["diff", "--cached", "--name-only"])?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn has_staged_changes(&self) -> Result<bool, GitError> {
        let changes = self.get_staged_changes()?;
        Ok(!changes.is_empty())
    }

    pub fn commit(&self, message: &str) -> Result<Output, GitError> {
        self.run_git_command(&["commit", "-m", message])
    }

    fn run_git_command(&self, args: &[&str]) -> Result<Output, GitError> {
        let mut command = Command::new("git");
        
        if let Some(dir) = &self.working_dir {
            command.current_dir(dir);
        }
        
        command.args(args);
        
        command.output().map_err(|e| GitError {
            message: format!("Git command failed: {}", e),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_test_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let git_client = GitClient::with_working_dir(temp_dir.path().to_string_lossy().to_string());
        
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
    fn test_has_staged_changes_with_no_changes() {
        let temp_dir = setup_test_repo();
        let git_client = GitClient::with_working_dir(temp_dir.path().to_string_lossy().to_string());
        
        assert!(!git_client.has_staged_changes().unwrap());
    }

    #[test]
    fn test_has_staged_changes_with_changes() {
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
            
        assert!(git_client.has_staged_changes().unwrap());
    }

    #[test]
    fn test_get_staged_files() {
        let temp_dir = setup_test_repo();
        let git_client = GitClient::with_working_dir(temp_dir.path().to_string_lossy().to_string());
        
        // Create and stage multiple test files
        let files = vec!["test1.txt", "test2.txt"];
        for file_name in &files {
            let test_file_path = temp_dir.path().join(file_name);
            let mut file = File::create(&test_file_path).unwrap();
            writeln!(file, "test content").unwrap();
            
            Command::new("git")
                .args(&["add", file_name])
                .current_dir(temp_dir.path())
                .output()
                .unwrap();
        }
        
        let staged_files = git_client.get_staged_files().unwrap();
        for file_name in &files {
            assert!(staged_files.contains(file_name));
        }
    }

    #[test]
    fn test_commit() {
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
            
        let commit_result = git_client.commit("test commit");
        assert!(commit_result.is_ok());
        
        // Verify commit was created
        let log_output = Command::new("git")
            .args(&["log", "--oneline"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();
        let log = String::from_utf8_lossy(&log_output.stdout);
        assert!(log.contains("test commit"));
    }
}
