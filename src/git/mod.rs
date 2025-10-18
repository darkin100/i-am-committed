use colored::*;
use log::info;
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

    pub fn get_staged_changes(&self) -> Result<String, GitError> {
        info!("GitClient::get_staged_changes called");
        let output = self.run_git_command(&["diff", "--cached", "--diff-algorithm=minimal"])?;
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        info!("GitClient::get_staged_changes completed:\n{}", result);
        Ok(result)
    }

    pub fn get_staged_files(&self) -> Result<String, GitError> {
        info!("GitClient::get_staged_files called");
        let output = self.run_git_command(&["diff", "--cached", "--name-only"])?;
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        info!(
            "GitClient::get_staged_files completed - {} files",
            result.lines().count()
        );
        Ok(result)
    }

    pub fn has_staged_changes(&self) -> Result<bool, GitError> {
        info!("GitClient::has_staged_changes called");
        let changes = self.get_staged_changes()?;
        let result = !changes.is_empty();
        info!(
            "GitClient::has_staged_changes completed - has changes: {}",
            result
        );
        Ok(result)
    }

    pub fn commit(&self, message: &str) -> Result<Output, GitError> {
        info!("GitClient::commit called with message: {}", message);
        let result = self.run_git_command(&["commit", "-m", message]);
        match &result {
            Ok(output) => info!(
                "GitClient::commit completed - success: {}",
                output.status.success()
            ),
            Err(e) => info!("GitClient::commit failed - error: {}", e),
        }
        result
    }

    fn run_git_command(&self, args: &[&str]) -> Result<Output, GitError> {
        info!("GitClient::run_git_command called with args: {:?}", args);
        let mut command = Command::new("git");

        if let Some(dir) = &self.working_dir {
            command.current_dir(dir);
        }

        command.args(args);

        let result = command.output().map_err(|e| GitError {
            message: format!("Git command failed: {}", e),
        });

        match &result {
            Ok(output) => info!(
                "GitClient::run_git_command completed - status: {}, stdout len: {}, stderr len: {}",
                output.status,
                output.stdout.len(),
                output.stderr.len()
            ),
            Err(e) => info!("GitClient::run_git_command failed - error: {}", e),
        }

        result
    }

    pub fn get_current_branch(&self) -> Result<String, GitError> {
        info!("GitClient::get_current_branch called");
        let output = self.run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!(
            "GitClient::get_current_branch completed - branch: {}",
            result
        );
        Ok(result)
    }

    pub fn get_commit_hash(&self) -> Result<String, GitError> {
        info!("GitClient::get_commit_hash called");
        let output = self.run_git_command(&["rev-parse", "--short", "HEAD"])?;
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!("GitClient::get_commit_hash completed - hash: {}", result);
        Ok(result)
    }

    pub fn commit_with_details(&self, commit_message: &str) -> Result<(), GitError> {
        info!(
            "GitClient::commit_with_details called with message: {}",
            commit_message
        );
        let output = self.commit(commit_message)?;

        if !output.status.success() {
            info!(
                "GitClient::commit_with_details - commit failed with status: {}",
                output.status
            );
            println!(
                "{} {}",
                "Failed to commit changes. Exit status:".red(),
                output.status
            );
            return Ok(());
        }

        let branch = self.get_current_branch()?;
        let commit = self.get_commit_hash()?;

        println!("\n✅ Commit Successful!");
        println!("-----------------------------------------");
        println!("🔹 Branch: {}", branch);
        println!("🔹 Commit: {}", commit);
        println!("🔹 Message: {}", commit_message);
        println!("-----------------------------------------");
        println!("🎉 All done! Keep up the great work!\n");

        info!(
            "GitClient::commit_with_details completed - branch: {}, commit: {}",
            branch, commit
        );
        Ok(())
    }

    pub fn get_file_content(&self, file_path: &str) -> Result<String, GitError> {
        info!(
            "GitClient::get_file_content called with file_path: {}",
            file_path
        );
        let output = self.run_git_command(&["show", &format!(":{}", file_path)])?;

        if !output.status.success() {
            let error = GitError {
                message: format!(
                    "Failed to get file content: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            };
            info!("GitClient::get_file_content failed - error: {}", error);
            return Err(error);
        }

        let result = String::from_utf8_lossy(&output.stdout).to_string();
        info!(
            "GitClient::get_file_content completed - content length: {} chars",
            result.len()
        );
        Ok(result)
    }

    pub fn get_file_diff(&self, file_path: &str) -> Result<String, GitError> {
        info!(
            "GitClient::get_file_diff called with file_path: {}",
            file_path
        );
        let output =
            self.run_git_command(&["diff", "--cached", "--diff-algorithm=minimal", file_path])?;
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        info!(
            "GitClient::get_file_diff completed - diff length: {} chars",
            result.len()
        );
        Ok(result)
    }

    pub fn get_commit_history(&self, count: usize) -> Result<String, GitError> {
        info!("GitClient::get_commit_history called with count: {}", count);
        let count_str = count.to_string();
        let output = self.run_git_command(&[
            "log",
            &format!("-{}", count_str),
            "--pretty=format:%h - %s (%an, %ar)",
        ])?;
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        info!(
            "GitClient::get_commit_history completed - {} commits returned",
            result.lines().count()
        );
        Ok(result)
    }

    pub fn list_staged_files_with_status(&self) -> Result<String, GitError> {
        info!("GitClient::list_staged_files_with_status called");
        let output = self.run_git_command(&["diff", "--cached", "--name-status"])?;
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        info!(
            "GitClient::list_staged_files_with_status completed - {} files",
            result.lines().count()
        );
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let _git_client =
            GitClient::with_working_dir(temp_dir.path().to_string_lossy().to_string());

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
