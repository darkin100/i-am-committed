use log::info;
use regex::Regex;
use std::path::PathBuf;

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

// Note: This test has been disabled because it relied on the old non-agentic workflow
// that accepted raw diffs as input. The new agentic workflow uses tools to fetch
// staged changes from git directly, making this historical evaluation test incompatible.
// To re-enable this test, you would need to either:
// 1. Create a test-only version of the API that accepts raw diffs, or
// 2. Refactor the test to work with actual git repositories
#[tokio::test]
#[ignore = "Disabled - requires old non-agentic API"]
async fn test_ai_agent_generates_conventional_commits() {
    println!("This test has been disabled. See comments in source for details.");
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
