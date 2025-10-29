use iamcommitted::agent::Agent;
use iamcommitted::ai::AIClient;
use iamcommitted::config::Config;
use iamcommitted::git::GitClient;
use regex::Regex;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Structure to hold parsed log data
#[derive(Debug)]
struct LogInteraction {
    diff: String,
    expected_message: String,
}

/// Parse a log file to extract the git diff and expected commit message
fn parse_log_file(content: &str) -> Option<LogInteraction> {
    // Extract the diff section (between "AI Request:" and "AI Response:")
    let request_regex =
        Regex::new(r"(?s)\[.*?\] \[INFO\] AI Request:\s*\n(.*?)\n\[.*?\] \[INFO\] AI Response:")
            .ok()?;
    let response_regex =
        Regex::new(r"(?s)\[.*?\] \[INFO\] AI Response:\s*\n(.*?)(?:\n\[.*?\]|$)").ok()?;

    let diff = request_regex
        .captures(content)?
        .get(1)?
        .as_str()
        .trim()
        .to_string();

    let expected_message = response_regex
        .captures(content)?
        .get(1)?
        .as_str()
        .trim()
        .to_string();

    if diff.is_empty() || expected_message.is_empty() {
        return None;
    }

    Some(LogInteraction {
        diff,
        expected_message,
    })
}

/// Load test data from log files in the data/split_logs directory
fn load_test_data_from_logs(limit: usize) -> Vec<(String, LogInteraction)> {
    let mut test_data = Vec::new();

    // Get the project root directory (parent of tests/)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let log_dir = PathBuf::from(manifest_dir).join("data").join("split_logs");

    if !log_dir.exists() {
        println!("Warning: Log directory not found: {:?}", log_dir);
        return test_data;
    }

    let entries = match fs::read_dir(&log_dir) {
        Ok(entries) => entries,
        Err(e) => {
            println!("Warning: Failed to read log directory: {}", e);
            return test_data;
        }
    };

    for entry in entries.flatten() {
        if test_data.len() >= limit {
            break;
        }

        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("log") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        if let Some(interaction) = parse_log_file(&content) {
            let filename = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            test_data.push((filename, interaction));
        }
    }

    test_data
}

/// Helper function to set up a test git repository with changes matching a diff
fn setup_test_repo_with_diff(diff: &str) -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Configure git user
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Parse the diff to extract file changes
    // This is a simplified approach - we'll create files mentioned in the diff
    let file_regex = Regex::new(r"diff --git [ci]/(.+?) [ci]/").unwrap();

    for caps in file_regex.captures_iter(diff) {
        if let Some(file_path_match) = caps.get(1) {
            let file_path = file_path_match.as_str();
            let full_path = temp_dir.path().join(file_path);

            // Create parent directories if needed
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).ok();
            }

            // Create a file with some content
            if let Ok(mut file) = fs::File::create(&full_path) {
                writeln!(file, "test content for {}", file_path).ok();
            }
        }
    }

    // Stage all created files
    Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    temp_dir
}

/// Helper function to check if OpenAI API key is available
fn get_api_key() -> Option<String> {
    env::var("IAC_OPENAI_API_KEY")
        .or_else(|_| env::var("OPENAI_API_KEY"))
        .ok()
}

/// Helper function to extract commit message from <commit_message> tags
fn extract_commit_message(raw_message: &str) -> String {
    let tag_regex = Regex::new(r"(?s)<commit_message>\s*(.*?)\s*</commit_message>").unwrap();

    if let Some(caps) = tag_regex.captures(raw_message) {
        caps.get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| raw_message.to_string())
    } else {
        raw_message.to_string()
    }
}

#[tokio::test]
async fn test_agent_with_real_diffs_matches_conventional_commits() {
    // Skip test if no API key is available
    let api_key = match get_api_key() {
        Some(key) => key,
        None => {
            println!("Skipping test: No OpenAI API key found");
            return;
        }
    };

    // Load test data from log files (limit to 50 for testing)
    let test_data = load_test_data_from_logs(50);

    if test_data.is_empty() {
        println!("Warning: No test data loaded from logs, skipping test");
        return;
    }

    // Regex pattern for conventional commit format
    // Based on analysis of actual commit messages in data/split_logs:
    // - Type: feat|fix|docs|style|refactor|perf|test|chore|build|ci|revert
    // - Scope (optional): alphanumeric, hyphens, underscores (e.g., ai, README, commit-options)
    // - Description: must start with lowercase letter, no period at end of first line
    let conventional_commit_regex = Regex::new(
        r"^(feat|fix|docs|style|refactor|perf|test|chore|build|ci|revert)(\([a-zA-Z0-9_-]+\))?: [a-z].+"
    )
    .unwrap();

    let mut passed = 0;
    let mut failed = 0;
    let mut total_prompt_tokens = 0;
    let mut total_completion_tokens = 0;
    let mut total_tokens = 0;

    for (filename, interaction) in test_data {
        println!("\n=== Testing: {} ===", filename);

        // Set up test repository
        let temp_dir = setup_test_repo_with_diff(&interaction.diff);
        let git_client = GitClient::with_working_dir(temp_dir.path().to_string_lossy().to_string());

        // Create AI client
        let config = Config::new().expect("Failed to create config");
        let mut ai_client =
            AIClient::new(api_key.clone(), config).expect("Failed to create AI client");

        println!("Using model: {}", ai_client.get_model());

        // Create agent
        let mut agent = Agent::new(&mut ai_client, &git_client).with_max_iterations(5);

        // Generate commit message
        let result = agent.generate_commit_message().await;

        if result.is_err() {
            println!("✗ Failed to generate message");
            failed += 1;
            continue;
        }

        let commit_result = result.unwrap();
        let commit_message = extract_commit_message(&commit_result.message);
        let first_line = commit_message.lines().next().unwrap_or("");

        // Track token usage
        total_prompt_tokens += commit_result.token_usage.prompt_tokens;
        total_completion_tokens += commit_result.token_usage.completion_tokens;
        total_tokens += commit_result.token_usage.total_tokens;

        // Check if it matches conventional commits format
        if conventional_commit_regex.is_match(first_line) {
            println!("✓ Matches conventional commits: {}", first_line);
            println!(
                "  Token usage: {} prompt + {} completion = {} total",
                commit_result.token_usage.prompt_tokens,
                commit_result.token_usage.completion_tokens,
                commit_result.token_usage.total_tokens
            );
            passed += 1;
        } else {
            println!("✗ Does NOT match conventional commits: {}", first_line);
            println!(
                "  Token usage: {} prompt + {} completion = {} total",
                commit_result.token_usage.prompt_tokens,
                commit_result.token_usage.completion_tokens,
                commit_result.token_usage.total_tokens
            );
            failed += 1;
        }
    }

    println!("\n=== Summary ===");
    println!("Passed: {}/{}", passed, passed + failed);
    println!("Failed: {}/{}", failed, passed + failed);
    println!(
        "Token Usage: {} prompt + {} completion = {} total",
        total_prompt_tokens, total_completion_tokens, total_tokens
    );

    // Assert that at least 80% pass
    let success_rate = passed as f64 / (passed + failed) as f64;
    assert!(
        success_rate >= 0.8,
        "Success rate ({:.1}%) should be at least 80%",
        success_rate * 100.0
    );
}

#[test]
fn test_log_file_parsing() {
    // Test that we can parse log files correctly
    let test_data = load_test_data_from_logs(10);

    if test_data.is_empty() {
        println!("Warning: No log files found in data/split_logs");
        println!("This test will pass but integration tests may not work");
        return;
    }

    println!("Successfully parsed {} log files", test_data.len());

    for (filename, interaction) in test_data.iter().take(3) {
        println!("\n=== {} ===", filename);
        println!(
            "Diff preview: {}",
            interaction
                .diff
                .lines()
                .take(3)
                .collect::<Vec<_>>()
                .join("\n")
        );
        println!(
            "Expected message: {}",
            interaction.expected_message.lines().next().unwrap_or("")
        );

        // Validate that we have actual content
        assert!(!interaction.diff.is_empty(), "Diff should not be empty");
        assert!(
            !interaction.expected_message.is_empty(),
            "Expected message should not be empty"
        );

        // Check that the diff looks like a git diff
        assert!(
            interaction.diff.contains("diff --git") || interaction.diff.contains("@@"),
            "Diff should contain git diff markers"
        );
    }
}
