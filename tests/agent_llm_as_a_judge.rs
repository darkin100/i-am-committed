use iamcommitted::agent::Agent;
use iamcommitted::ai::AIClient;
use iamcommitted::config::Config;
use iamcommitted::git::GitClient;
use openai_api_rs::v1::chat_completion::{ChatCompletionMessage, Content, MessageRole};
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

/// Structure to hold judge evaluation results
#[derive(Debug)]
struct JudgeEvaluation {
    conventional_commits_score: u8, // 0-10
    clarity_score: u8,              // 0-10
    accuracy_score: u8,             // 0-10
    overall_score: u8,              // 0-10
    reasoning: String,
    passes: bool,
}

/// Parse the judge's evaluation response
fn parse_judge_response(response: &str) -> Option<JudgeEvaluation> {
    // Extract scores using regex
    let conv_regex = Regex::new(r"Conventional Commits Score:\s*(\d+)").ok()?;
    let clarity_regex = Regex::new(r"Clarity Score:\s*(\d+)").ok()?;
    let accuracy_regex = Regex::new(r"Accuracy Score:\s*(\d+)").ok()?;
    let overall_regex = Regex::new(r"Overall Score:\s*(\d+)").ok()?;
    let passes_regex = Regex::new(r"Passes:\s*(Yes|No)").ok()?;

    let conventional_commits_score = conv_regex
        .captures(response)?
        .get(1)?
        .as_str()
        .parse::<u8>()
        .ok()?;

    let clarity_score = clarity_regex
        .captures(response)?
        .get(1)?
        .as_str()
        .parse::<u8>()
        .ok()?;

    let accuracy_score = accuracy_regex
        .captures(response)?
        .get(1)?
        .as_str()
        .parse::<u8>()
        .ok()?;

    let overall_score = overall_regex
        .captures(response)?
        .get(1)?
        .as_str()
        .parse::<u8>()
        .ok()?;

    let passes = passes_regex.captures(response)?.get(1)?.as_str() == "Yes";

    // Extract reasoning (everything after "Reasoning:")
    let reasoning_regex = Regex::new(r"(?s)Reasoning:\s*(.+)").ok()?;
    let reasoning = reasoning_regex
        .captures(response)?
        .get(1)?
        .as_str()
        .trim()
        .to_string();

    Some(JudgeEvaluation {
        conventional_commits_score,
        clarity_score,
        accuracy_score,
        overall_score,
        reasoning,
        passes,
    })
}

/// Use LLM as a judge to evaluate commit message quality
async fn evaluate_commit_message_with_llm(
    ai_client: &mut AIClient,
    diff: &str,
    commit_message: &str,
) -> Result<JudgeEvaluation, Box<dyn std::error::Error>> {
    let judge_prompt = format!(
        r#"You are an expert code reviewer evaluating Git commit messages. Your task is to evaluate the quality of a commit message based on the git diff provided.

Evaluation Criteria:
1. **Conventional Commits Format** (0-10): Does it follow the format "type(scope): description"?
   - Valid types: feat, fix, docs, style, refactor, perf, test, chore, build, ci, revert
   - Description should start with lowercase
   - No period at the end of the first line

2. **Clarity** (0-10): Is the message clear, concise, and understandable?
   - Does it clearly communicate what changed?
   - Is it written in imperative mood?
   - Is it concise but descriptive?

3. **Accuracy** (0-10): Does the message accurately reflect the changes in the diff?
   - Does the type match the changes (feat for new features, fix for bugs, etc.)?
   - Does the scope (if present) make sense?
   - Does the description match what actually changed?

4. **Overall Score** (0-10): Your overall assessment of the commit message quality.

Git Diff:
```
{}
```

Generated Commit Message:
```
{}
```

Please provide your evaluation in the following format:

Conventional Commits Score: [0-10]
Clarity Score: [0-10]
Accuracy Score: [0-10]
Overall Score: [0-10]
Passes: [Yes/No] (Yes if overall score >= 8)
Reasoning: [Your detailed explanation of the scores]
"#,
        diff, commit_message
    );

    // Create messages for the judge
    let messages = vec![
        ChatCompletionMessage {
            role: MessageRole::system,
            content: Content::Text("You are an expert code reviewer evaluating Git commit messages. Provide structured evaluations following the exact format requested.".to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        },
        ChatCompletionMessage {
            role: MessageRole::user,
            content: Content::Text(judge_prompt),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    let judge_response = ai_client.simple_chat_completion(messages).await?;

    parse_judge_response(&judge_response).ok_or_else(|| "Failed to parse judge response".into())
}

#[tokio::test]
async fn test_agent_with_real_diffs_using_llm_judge() {
    // Skip test if no API key is available
    let api_key = match get_api_key() {
        Some(key) => key,
        None => {
            println!("Skipping test: No OpenAI API key found");
            return;
        }
    };

    // Load test data from log files (limit to 10 for testing due to API costs)
    let test_data = load_test_data_from_logs(10);

    if test_data.is_empty() {
        println!("Warning: No test data loaded from logs, skipping test");
        return;
    }

    let mut passed = 0;
    let mut failed = 0;
    let mut total_prompt_tokens = 0;
    let mut total_completion_tokens = 0;
    let mut total_tokens = 0;
    let mut total_conventional_score = 0u32;
    let mut total_clarity_score = 0u32;
    let mut total_accuracy_score = 0u32;
    let mut total_overall_score = 0u32;

    for (filename, interaction) in test_data.iter() {
        println!("\n=== Testing: {} ===", filename);

        // Set up test repository
        let temp_dir = setup_test_repo_with_diff(&interaction.diff);
        let git_client = GitClient::with_working_dir(temp_dir.path().to_string_lossy().to_string());

        // Create AI client for generation
        let config = Config::new().expect("Failed to create config");
        let mut ai_client =
            AIClient::new(api_key.clone(), config).expect("Failed to create AI client");

        println!("Using model: {}", ai_client.get_model());

        // Create agent
        let mut agent = Agent::new(&mut ai_client, &git_client).with_max_iterations(5);

        // Generate commit message
        let result = agent.generate_commit_message().await;

        if result.is_err() {
            println!("✗ Failed to generate message: {:?}", result.err());
            failed += 1;
            continue;
        }

        let commit_result = result.unwrap();
        let commit_message = extract_commit_message(&commit_result.message);
        let first_line = commit_message.lines().next().unwrap_or("");

        println!("Generated message: {}", first_line);

        // Track token usage from generation
        total_prompt_tokens += commit_result.token_usage.prompt_tokens;
        total_completion_tokens += commit_result.token_usage.completion_tokens;
        total_tokens += commit_result.token_usage.total_tokens;

        // Create a new AI client for the judge (to keep contexts separate)
        let config = Config::new().expect("Failed to create config");
        let mut judge_client =
            AIClient::new(api_key.clone(), config).expect("Failed to create judge AI client");

        // Evaluate using LLM as judge
        let evaluation =
            evaluate_commit_message_with_llm(&mut judge_client, &interaction.diff, &commit_message)
                .await;

        match evaluation {
            Ok(eval) => {
                total_conventional_score += eval.conventional_commits_score as u32;
                total_clarity_score += eval.clarity_score as u32;
                total_accuracy_score += eval.accuracy_score as u32;
                total_overall_score += eval.overall_score as u32;

                if eval.passes {
                    println!("✓ PASSED (Overall: {}/10)", eval.overall_score);
                    passed += 1;
                } else {
                    println!("✗ FAILED (Overall: {}/10)", eval.overall_score);
                    failed += 1;
                }

                println!(
                    "  Conventional Commits: {}/10",
                    eval.conventional_commits_score
                );
                println!("  Clarity: {}/10", eval.clarity_score);
                println!("  Accuracy: {}/10", eval.accuracy_score);
                println!(
                    "  Reasoning: {}",
                    eval.reasoning.lines().next().unwrap_or("")
                );
                println!(
                    "  Token usage: {} prompt + {} completion = {} total",
                    commit_result.token_usage.prompt_tokens,
                    commit_result.token_usage.completion_tokens,
                    commit_result.token_usage.total_tokens
                );
            }
            Err(e) => {
                println!("✗ Judge evaluation failed: {}", e);
                failed += 1;
            }
        }
    }

    let total_tests = passed + failed;

    println!("\n=== Summary ===");
    println!("Passed: {}/{}", passed, total_tests);
    println!("Failed: {}/{}", failed, total_tests);

    if total_tests > 0 {
        println!(
            "Average Scores: Conv={:.1} Clarity={:.1} Accuracy={:.1} Overall={:.1}",
            total_conventional_score as f64 / total_tests as f64,
            total_clarity_score as f64 / total_tests as f64,
            total_accuracy_score as f64 / total_tests as f64,
            total_overall_score as f64 / total_tests as f64
        );
    }

    println!(
        "Token Usage: {} prompt + {} completion = {} total",
        total_prompt_tokens, total_completion_tokens, total_tokens
    );

    // Assert that at least 80% pass
    let success_rate = passed as f64 / total_tests as f64;
    assert!(
        success_rate >= 0.8,
        "Success rate ({:.1}%) should be at least 80%",
        success_rate * 100.0
    );
}
