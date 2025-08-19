use super::types::CommitType;
use once_cell::sync::Lazy;
use regex::Regex;

// Compile regex patterns once for better performance
static CODE_BLOCK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)^\s*```(?:shell|sh|bash|plaintext|text|markdown|md)\s*\n(.*?)\n```\s*$")
        .unwrap()
});

static SIMPLE_CODE_BLOCK_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)^\s*```\s*\n(.*?)\n```\s*$").unwrap());

static INLINE_KEYWORD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:shell|sh|bash|plaintext|text|markdown|md)\s+").unwrap());

pub struct CommitFormatter {
    raw_message: String,
}

impl CommitFormatter {
    pub fn new(raw_message: String) -> Self {
        CommitFormatter { raw_message }
    }

    pub fn format(&self) -> CommitType {
        let mut cleaned_message = self.raw_message.clone();

        // Remove commit_message XML tags
        cleaned_message = cleaned_message
            .replace("<commit_message>", "")
            .replace("</commit_message>", "");

        // Remove markdown code blocks with language identifiers that wrap the entire message
        if let Some(captures) = CODE_BLOCK_REGEX.captures(&cleaned_message) {
            if let Some(content) = captures.get(1) {
                cleaned_message = content.as_str().to_string();
            }
        }

        // Also handle code blocks without language identifiers but only if they wrap the entire message
        if let Some(captures) = SIMPLE_CODE_BLOCK_REGEX.captures(&cleaned_message) {
            if let Some(content) = captures.get(1) {
                cleaned_message = content.as_str().to_string();
            }
        }

        // Remove inline language keywords at the start of the message
        cleaned_message = INLINE_KEYWORD_REGEX
            .replace(&cleaned_message, "")
            .to_string();

        // Remove remaining backticks (for backward compatibility)
        cleaned_message = cleaned_message.replace("`", "");

        // Trim whitespace
        cleaned_message = cleaned_message.trim().to_string();

        CommitType::new(cleaned_message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_message() {
        let message = "feat(auth): Add login functionality";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(format!("{}", formatter.format()), message);
    }

    #[test]
    fn test_format_message_with_backticks() {
        let message = "```feat(auth): Add login functionality```";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "feat(auth): Add login functionality"
        );
    }

    #[test]
    fn test_format_message_with_single_backticks() {
        let message = "`feat(core)`: Add new feature";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "feat(core): Add new feature"
        );
    }

    #[test]
    fn test_format_message_with_multiline_backticks() {
        let message = "feat(core): Add new feature
        ```
        added new feature
        ```";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "feat(core): Add new feature
        
        added new feature"
        );
    }

    #[test]
    fn test_format_message_with_commit_message_tags() {
        let message = "<commit_message>feat(auth): Add login functionality</commit_message>";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "feat(auth): Add login functionality"
        );
    }

    #[test]
    fn test_format_message_with_commit_message_tags_and_backticks() {
        let message = "<commit_message>`feat(core)`: Add new feature</commit_message>";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "feat(core): Add new feature"
        );
    }

    #[test]
    fn test_format_message_with_multiline_commit_message_tags() {
        let message = "<commit_message>
feat(core): Add new feature

Added a new feature to the core module
</commit_message>";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "feat(core): Add new feature

Added a new feature to the core module"
        );
    }

    #[test]
    fn test_format_message_with_shell_keyword() {
        let message = "shell feat(formatter): Update test assertions for CommitFormatter";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "feat(formatter): Update test assertions for CommitFormatter"
        );
    }

    #[test]
    fn test_format_message_with_sh_keyword() {
        let message = "sh fix(api): Resolve connection timeout issue";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "fix(api): Resolve connection timeout issue"
        );
    }

    #[test]
    fn test_format_message_with_plaintext_keyword() {
        let message = "plaintext refactor(core): Simplify error handling logic";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "refactor(core): Simplify error handling logic"
        );
    }

    #[test]
    fn test_format_message_with_bash_keyword() {
        let message = "bash chore(deps): Update dependencies to latest versions";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "chore(deps): Update dependencies to latest versions"
        );
    }

    #[test]
    fn test_format_message_with_text_keyword() {
        let message = "text docs(readme): Add installation instructions";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "docs(readme): Add installation instructions"
        );
    }

    #[test]
    fn test_format_message_with_markdown_code_block_shell() {
        let message = "```shell
feat(auth): Add OAuth2 authentication support
```";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "feat(auth): Add OAuth2 authentication support"
        );
    }

    #[test]
    fn test_format_message_with_markdown_code_block_sh() {
        let message = "```sh
fix(parser): Handle edge case in JSON parsing
```";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "fix(parser): Handle edge case in JSON parsing"
        );
    }

    #[test]
    fn test_format_message_with_markdown_code_block_plaintext() {
        let message = "```plaintext
perf(cache): Optimize cache invalidation strategy
```";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "perf(cache): Optimize cache invalidation strategy"
        );
    }

    #[test]
    fn test_format_message_with_markdown_code_block_multiline() {
        let message = "```shell
feat(ui): Add dark mode support

- Implemented theme switching functionality
- Added dark color palette
- Updated component styles
```";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "feat(ui): Add dark mode support

- Implemented theme switching functionality
- Added dark color palette
- Updated component styles"
        );
    }

    #[test]
    fn test_format_message_with_combined_patterns() {
        let message = "<commit_message>```shell
feat(api): Add rate limiting

Implemented rate limiting to prevent API abuse
```</commit_message>";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            format!("{}", formatter.format()),
            "feat(api): Add rate limiting

Implemented rate limiting to prevent API abuse"
        );
    }

    #[test]
    fn test_real_llm_examples_from_issue() {
        // Example 1 from GitHub issue #18
        let example1 = "shell\nfeat(formatter): Update test assertions for CommitFormatter\n\nBody:\n- Updated test assertions in CommitFormatter::format() to use format!(\"{}\", formatter.format()) instead of formatter.format().to_string().\n- Refactored CommitType struct to implement fmt::Display trait, replacing the previous to_string method.\n- Updated the configuration file prompts.md with new example (chore) and updated user prompt section for better instructions.";
        let formatter1 = CommitFormatter::new(example1.to_string());
        let result1 = format!("{}", formatter1.format());
        assert!(result1.starts_with("feat(formatter): Update test assertions for CommitFormatter"));
        assert!(!result1.starts_with("shell"));

        // Example 2 from GitHub issue #18
        let example2 = "sh\nfeat(formatter): Update test assertions for CommitFormatter";
        let formatter2 = CommitFormatter::new(example2.to_string());
        let result2 = format!("{}", formatter2.format());
        assert_eq!(
            result2,
            "feat(formatter): Update test assertions for CommitFormatter"
        );
        assert!(!result2.starts_with("sh"));

        // Additional realistic examples
        let example3 = "```plaintext\nfix(api): Handle null response gracefully\n\n- Added null checks in response handler\n- Improved error messages\n```";
        let formatter3 = CommitFormatter::new(example3.to_string());
        let result3 = format!("{}", formatter3.format());
        assert!(result3.starts_with("fix(api): Handle null response gracefully"));
        assert!(!result3.contains("```"));
        assert!(!result3.contains("plaintext"));
    }
}
