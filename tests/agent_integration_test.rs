use iamcommitted::agent::{Agent, AgentError};
use iamcommitted::ai::AIClient;
use iamcommitted::config::Config;
use iamcommitted::git::GitClient;
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

/// Helper function to set up a test git repository with staged changes
fn setup_test_repo_with_changes() -> TempDir {
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

    // Create and stage a test file
    let test_file_path = temp_dir.path().join("test.txt");
    let mut file = File::create(&test_file_path).unwrap();
    writeln!(file, "test content").unwrap();

    Command::new("git")
        .args(&["add", "test.txt"])
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

#[tokio::test]
async fn test_agent_generates_valid_commit_message() {
    // Skip test if no API key is available
    let api_key = match get_api_key() {
        Some(key) => key,
        None => {
            println!("Skipping test: No OpenAI API key found");
            return;
        }
    };

    // Set up test repository
    let temp_dir = setup_test_repo_with_changes();
    let git_client = GitClient::with_working_dir(temp_dir.path().to_string_lossy().to_string());

    // Create AI client
    let config = Config::new().expect("Failed to create config");
    let mut ai_client = AIClient::new(api_key, config).expect("Failed to create AI client");

    // Create agent
    let mut agent = Agent::new(&mut ai_client, &git_client).with_max_iterations(5);

    // Generate commit message
    let result = agent.generate_commit_message().await;

    // Assert that we got a result
    assert!(result.is_ok(), "Agent should successfully generate a commit message");

    let commit_result = result.unwrap();

    // Verify the message is not empty
    assert!(!commit_result.message.is_empty(), "Commit message should not be empty");

    // Verify token usage is tracked
    assert!(commit_result.token_usage.total_tokens > 0, "Should have token usage");
    assert!(commit_result.token_usage.prompt_tokens > 0, "Should have prompt tokens");
    assert!(commit_result.token_usage.completion_tokens > 0, "Should have completion tokens");

    println!("Generated commit message: {}", commit_result.message);
    println!(
        "Token usage - Prompt: {}, Completion: {}, Total: {}",
        commit_result.token_usage.prompt_tokens,
        commit_result.token_usage.completion_tokens,
        commit_result.token_usage.total_tokens
    );
}

#[tokio::test]
async fn test_agent_message_matches_conventional_commits_format() {
    // Skip test if no API key is available
    let api_key = match get_api_key() {
        Some(key) => key,
        None => {
            println!("Skipping test: No OpenAI API key found");
            return;
        }
    };

    // Set up test repository
    let temp_dir = setup_test_repo_with_changes();
    let git_client = GitClient::with_working_dir(temp_dir.path().to_string_lossy().to_string());

    // Create AI client
    let config = Config::new().expect("Failed to create config");
    let mut ai_client = AIClient::new(api_key, config).expect("Failed to create AI client");

    // Create agent
    let mut agent = Agent::new(&mut ai_client, &git_client).with_max_iterations(5);

    // Generate commit message
    let result = agent.generate_commit_message().await;
    assert!(result.is_ok(), "Agent should successfully generate a commit message");

    let commit_message = result.unwrap().message;

    // Test regex pattern for conventional commit format
    // Format: <type>(<scope>): <description>
    // or: <type>: <description>
    let conventional_commit_regex = Regex::new(
        r"^(feat|fix|docs|style|refactor|perf|test|chore|build|ci|revert)(\(.+\))?: .+"
    )
    .unwrap();

    assert!(
        conventional_commit_regex.is_match(&commit_message.lines().next().unwrap_or("")),
        "Commit message should match conventional commits format. Got: {}",
        commit_message
    );

    println!("✓ Commit message matches conventional commits format");
    println!("Generated message: {}", commit_message);
}

#[tokio::test]
async fn test_agent_message_has_proper_structure() {
    // Skip test if no API key is available
    let api_key = match get_api_key() {
        Some(key) => key,
        None => {
            println!("Skipping test: No OpenAI API key found");
            return;
        }
    };

    // Set up test repository with more substantial changes
    let temp_dir = TempDir::new().unwrap();

    // Initialize git repo
    Command::new("git")
        .args(&["init"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Configure git
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

    // Create a more substantial change
    let test_file_path = temp_dir.path().join("feature.rs");
    let mut file = File::create(&test_file_path).unwrap();
    writeln!(file, "pub fn new_feature() {{").unwrap();
    writeln!(file, "    println!(\"New feature implementation\");").unwrap();
    writeln!(file, "}}").unwrap();

    Command::new("git")
        .args(&["add", "feature.rs"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let git_client = GitClient::with_working_dir(temp_dir.path().to_string_lossy().to_string());

    // Create AI client
    let config = Config::new().expect("Failed to create config");
    let mut ai_client = AIClient::new(api_key, config).expect("Failed to create AI client");

    // Create agent
    let mut agent = Agent::new(&mut ai_client, &git_client).with_max_iterations(5);

    // Generate commit message
    let result = agent.generate_commit_message().await;
    assert!(result.is_ok(), "Agent should successfully generate a commit message");

    let commit_message = result.unwrap().message;

    // Verify structure: should have a subject line
    let lines: Vec<&str> = commit_message.lines().collect();
    assert!(!lines.is_empty(), "Should have at least a subject line");

    let subject = lines[0];

    // Subject line should not be too long (conventional limit is 50-72 chars)
    assert!(
        subject.len() <= 100,
        "Subject line should be reasonably short (<=100 chars). Got: {}",
        subject.len()
    );

    // Subject should not end with a period
    assert!(
        !subject.ends_with('.'),
        "Subject line should not end with a period"
    );

    // Subject should start with a lowercase word after the type:
    let subject_description_regex = Regex::new(r"^[a-z]+(\(.+\))?: [a-z]").unwrap();
    assert!(
        subject_description_regex.is_match(subject),
        "Subject description should start with lowercase. Got: {}",
        subject
    );

    println!("✓ Commit message has proper structure");
    println!("Subject: {}", subject);
}

#[tokio::test]
async fn test_agent_handles_no_staged_changes() {
    // Skip test if no API key is available
    let api_key = match get_api_key() {
        Some(key) => key,
        None => {
            println!("Skipping test: No OpenAI API key found");
            return;
        }
    };

    // Set up empty test repository
    let temp_dir = TempDir::new().unwrap();

    Command::new("git")
        .args(&["init"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

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

    let git_client = GitClient::with_working_dir(temp_dir.path().to_string_lossy().to_string());

    // Verify no staged changes
    assert!(!git_client.has_staged_changes().unwrap());

    // Create AI client
    let config = Config::new().expect("Failed to create config");
    let mut ai_client = AIClient::new(api_key, config).expect("Failed to create AI client");

    // Create agent
    let mut agent = Agent::new(&mut ai_client, &git_client).with_max_iterations(5);

    // Generate commit message - should still work but indicate no changes
    let result = agent.generate_commit_message().await;

    // The agent should handle this gracefully
    // It might return an error or a message indicating no changes
    match result {
        Ok(commit_result) => {
            println!("Agent returned message: {}", commit_result.message);
            // Message should indicate no changes or be empty/minimal
            let lowercase_msg = commit_result.message.to_lowercase();
            assert!(
                lowercase_msg.contains("no")
                    || lowercase_msg.contains("empty")
                    || lowercase_msg.contains("nothing")
                    || commit_result.message.is_empty(),
                "Message should indicate no changes. Got: {}",
                commit_result.message
            );
        }
        Err(e) => {
            println!("Agent returned error (expected): {}", e);
            // This is also acceptable behavior
        }
    }
}

#[tokio::test]
async fn test_agent_respects_max_iterations() {
    // This test verifies that the agent respects the max_iterations limit
    // Skip test if no API key is available
    let api_key = match get_api_key() {
        Some(key) => key,
        None => {
            println!("Skipping test: No OpenAI API key found");
            return;
        }
    };

    let temp_dir = setup_test_repo_with_changes();
    let git_client = GitClient::with_working_dir(temp_dir.path().to_string_lossy().to_string());

    let config = Config::new().expect("Failed to create config");
    let mut ai_client = AIClient::new(api_key, config).expect("Failed to create AI client");

    // Set a very low max iterations - agent should still handle this gracefully
    let mut agent = Agent::new(&mut ai_client, &git_client).with_max_iterations(1);

    let result = agent.generate_commit_message().await;

    // Should either succeed quickly or fail with max iterations error
    match result {
        Ok(commit_result) => {
            println!("✓ Agent succeeded within 1 iteration");
            println!("Message: {}", commit_result.message);
            assert!(!commit_result.message.is_empty());
        }
        Err(e) => {
            // If it fails, it should be due to max iterations
            println!("Agent failed as expected with low max_iterations: {}", e);
            assert!(
                e.message.contains("maximum iterations")
                    || e.message.contains("max iterations"),
                "Error should mention max iterations. Got: {}",
                e.message
            );
        }
    }
}
