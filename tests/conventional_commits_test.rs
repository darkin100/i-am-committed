use iamcommitted::ai::AIClient;
use iamcommitted::config::Config;
use iamcommitted::commit_formatter::CommitFormatter;
use log::{info, warn};
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::env;

/// Setup logging for tests - writes to tests/test.log
fn setup_test_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Use std::sync::Once to ensure we only initialize once
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let log_path = PathBuf::from("tests/test.log");

        let dispatch = fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{}] [{}] {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    message
                ))
            })
            .level(log::LevelFilter::Info)
            .chain(fern::log_file(&log_path).unwrap());

        dispatch.apply().ok();

        println!("📝 Test logs will be written to: {}", log_path.display());
    });

    Ok(())
}

/// Conventional Commits regex pattern
/// Format: type(optional-scope): description
///
/// Valid types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
/// Scope can contain: letters, numbers, spaces, hyphens, underscores, dots, slashes
/// Examples:
/// - feat(api): add new endpoint
/// - fix: correct typo in documentation
/// - docs(readme): update installation instructions
/// - docs(Routine Tasks): add documentation (spaces and capitals allowed)
fn get_conventional_commit_regex() -> Regex {
    Regex::new(
        r"^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\([a-zA-Z0-9_\-\s./]+\))?: .+",
    )
    .unwrap()
}

/// Parse a log file and extract the git diff from AI Request
///
/// Log files contain lines like:
/// [2025-03-20 21:05:45] [INFO] AI Request:
/// <git diff content>
/// [2025-03-20 21:05:45] [INFO] AI Response:
/// <commit message>
///
/// This function extracts the git diff between AI Request and AI Response markers
fn extract_git_diff(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut diff_lines = Vec::new();
    let mut capturing = false;

    for line in lines {
        if line.contains("[INFO] AI Request:") {
            capturing = true;
            continue;
        }

        if line.contains("[INFO] AI Response:") {
            break;
        }

        if capturing {
            diff_lines.push(line);
        }
    }

    if diff_lines.is_empty() {
        None
    } else {
        Some(diff_lines.join("\n"))
    }
}

/// Extract the first line of the commit message from AI response
fn extract_first_line(message: &str) -> String {
    message.lines().next().unwrap_or("").trim().to_string()
}

#[test]
fn test_conventional_commits_regex() {
    // Initialize logging for this test
    setup_test_logging().ok();

    info!("Testing conventional commits regex patterns");

    let regex = get_conventional_commit_regex();

    // Valid commit messages
    assert!(regex.is_match("feat(api): add new endpoint"));
    assert!(regex.is_match("fix: correct typo in documentation"));
    assert!(regex.is_match("docs(readme): update installation instructions"));
    assert!(regex.is_match("refactor(core): simplify logic"));
    assert!(regex.is_match("chore: update dependencies"));
    assert!(regex.is_match("test(unit): add test for parser"));
    assert!(regex.is_match("docs(Routine Tasks): add documentation"));
    assert!(regex.is_match("docs(Local Development): expand warnings"));
    assert!(regex.is_match("chore(.gitignore): update ignored files"));

    info!("All valid commit message patterns matched successfully");

    // Invalid commit messages
    assert!(!regex.is_match("Add new feature"));
    assert!(!regex.is_match("feature: add something"));
    assert!(!regex.is_match("feat:missing space"));
    assert!(!regex.is_match("feat(scope) missing colon"));

    info!("All invalid commit message patterns rejected successfully");
}

#[tokio::test]
async fn test_ai_agent_generates_conventional_commits() {
    // Initialize logging
    setup_test_logging().ok();

    info!("=== Starting AI Agent Conventional Commits Test ===");

    // Check for API key - skip test if not available
    let api_key = match env::var("IAC_OPENAI_API_KEY")
        .or_else(|_| env::var("OPENAI_API_KEY"))
    {
        Ok(key) => key,
        Err(_) => {
            warn!("Skipping test: No OpenAI API key found");
            warn!("Set IAC_OPENAI_API_KEY or OPENAI_API_KEY to run this test");
            println!("⚠️  Skipping test: No OpenAI API key found");
            println!("   Set IAC_OPENAI_API_KEY or OPENAI_API_KEY to run this test");
            return;
        }
    };

    let split_logs_dir = PathBuf::from("data/split_logs");

    // Check if the directory exists
    if !split_logs_dir.exists() {
        panic!(
            "Split logs directory not found at: {}",
            split_logs_dir.display()
        );
    }

    info!("Split logs directory found: {}", split_logs_dir.display());

    // Initialize AI client
    let config = Config::new().expect("Failed to create config");
    let ai_client = AIClient::new(api_key, config).expect("Failed to create AI client");
    let regex = get_conventional_commit_regex();

    info!("AI client initialized successfully");

    let mut total_files = 0;
    let mut valid_responses = 0;
    let mut invalid_responses = Vec::new();
    let mut skipped_files = Vec::new();

    // Read all log files
    let entries = fs::read_dir(&split_logs_dir).expect("Failed to read split_logs directory");

    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        // Only process .log files
        if path.extension().and_then(|s| s.to_str()) != Some("log") {
            continue;
        }

        total_files += 1;
        let filename = path.file_name().unwrap().to_string_lossy().to_string();

        info!("Processing file {}/{}: {}", total_files, 133, filename);

        // Read the log file
        let content = fs::read_to_string(&path)
            .expect(&format!("Failed to read file: {}", path.display()));

        // Extract git diff
        let diff = match extract_git_diff(&content) {
            Some(d) => d,
            None => {
                warn!("Skipping {}: Could not extract git diff", filename);
                println!("⚠️  Skipping {}: Could not extract git diff", filename);
                skipped_files.push(filename);
                continue;
            }
        };

        info!("Extracted git diff from {} ({} bytes)", filename, diff.len());

        // Generate commit message using AI agent
        println!("🤖 Testing {} ({}/{})", filename, total_files, 133);

        match ai_client.generate_commit_message(&diff).await {
            Ok(raw_message) => {
                // Format the message
                let formatter = CommitFormatter::new(raw_message.clone());
                let formatted = formatter.format();
                let first_line = extract_first_line(&formatted.to_string());

                // Validate against conventional commits format
                if regex.is_match(&first_line) {
                    valid_responses += 1;
                    info!("✅ Valid commit message for {}: {}", filename, first_line);
                    println!("   ✅ Valid: {}", first_line);
                } else {
                    invalid_responses.push((filename.clone(), first_line.clone()));
                    warn!("❌ Invalid commit message for {}: {}", filename, first_line);
                    println!("   ❌ Invalid: {}", first_line);
                }
            }
            Err(e) => {
                warn!("API Error for {}: {}", filename, e);
                println!("   ⚠️  API Error: {}", e);
                skipped_files.push(filename);
            }
        }

        // Add a small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Print and log summary
    info!("=== Test Execution Complete ===");
    info!("Total log files: {}", total_files);
    info!("Valid responses: {}", valid_responses);
    info!("Invalid responses: {}", invalid_responses.len());
    info!("Skipped/Errors: {}", skipped_files.len());

    println!("\n=== AI Agent Conventional Commits Test Summary ===");
    println!("Total log files: {}", total_files);
    println!("Valid responses: {}", valid_responses);
    println!("Invalid responses: {}", invalid_responses.len());
    println!("Skipped/Errors: {}", skipped_files.len());

    if !invalid_responses.is_empty() {
        info!("=== Invalid Responses ===");
        println!("\n=== Invalid Responses ===");
        for (filename, response) in &invalid_responses {
            info!("File: {} | Response: {}", filename, response);
            println!("\nFile: {}", filename);
            println!("Response: {}", response);
        }
    }

    // Calculate success rate
    let tested = valid_responses + invalid_responses.len();
    if tested > 0 {
        let success_rate = (valid_responses as f64 / tested as f64) * 100.0;
        info!("✨ Success Rate: {:.1}% ({}/{})", success_rate, valid_responses, tested);
        println!("\n✨ Success Rate: {:.1}%", success_rate);
    }

    info!("=== Test Summary Complete ===");

    // Assert that at least 90% of responses follow conventional commits format
    let tested = valid_responses + invalid_responses.len();
    if tested > 0 {
        let success_rate = (valid_responses as f64 / tested as f64) * 100.0;
        assert!(
            success_rate >= 90.0,
            "\nSuccess rate {:.1}% is below 90% threshold. {} out of {} responses are invalid.",
            success_rate,
            invalid_responses.len(),
            tested
        );
    }
}

#[test]
fn test_extract_git_diff() {
    let sample_log = r#"[2025-03-20 21:05:45] [INFO] AI Request:
diff --git c/readme.md i/readme.md
index 851ec2d..a8a2548 100644
--- c/readme.md
+++ i/readme.md
@@ -1,3 +1,4 @@
+# New Title
 # README

[2025-03-20 21:05:45] [INFO] AI Response:
docs(readme): add new title to documentation

Additional description here.
"#;

    let diff = extract_git_diff(sample_log);
    assert!(diff.is_some());

    let diff_content = diff.unwrap();
    assert!(diff_content.contains("diff --git"));
    assert!(diff_content.contains("+# New Title"));
    assert!(!diff_content.contains("[INFO] AI Response:"));
}
