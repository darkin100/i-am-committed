use regex::Regex;
use super::types::{COMMIT_TYPES, CommitType};

pub struct CommitFormatter {
    raw_message: String,
}

impl CommitFormatter {
    pub fn new(raw_message: String) -> Self {
        CommitFormatter { raw_message }
    }

    pub fn format(&self) -> CommitType {
        let type_str = self.determine_type();
        let scope = self.extract_scope();
        let description = self.format_description();

        CommitType::new(
            type_str.to_string(),
            scope,
            description,
        )
    }

    fn determine_type(&self) -> &'static str {
        let lower_message = self.raw_message.to_lowercase();

        // Check for common patterns indicating different types
        for &commit_type in COMMIT_TYPES {
            let patterns = match commit_type {
                "feat" => vec!["add", "new", "implement", "create"],
                "fix" => vec!["fix", "bug", "issue", "resolve", "patch"],
                "docs" => vec!["document", "comment", "readme"],
                "style" => vec!["format", "style", "lint"],
                "refactor" => vec!["refactor", "restructure", "reorganize"],
                "perf" => vec!["performance", "optimize", "speed"],
                "test" => vec!["test", "spec", "coverage"],
                "build" => vec!["build", "dependency", "version"],
                "ci" => vec!["ci", "pipeline", "workflow", "github action"],
                "revert" => vec!["revert", "rollback", "undo"],
                _ => vec![],
            };

            for pattern in patterns {
                if lower_message.contains(pattern) {
                    return commit_type;
                }
            }
        }

        // Default to "chore" if no specific type is identified
        "chore"
    }

    fn extract_scope(&self) -> Option<String> {
        // Look for common scope patterns in the message
        let scope_patterns = [
            // Match explicit scope indicators
            Regex::new(r"in\s+the\s+([a-zA-Z0-9_-]+)\s+component").ok()?,
            Regex::new(r"in\s+([a-zA-Z0-9_-]+)\s+module").ok()?,
            // Match file paths that might indicate scope
            Regex::new(r"(?:^|\s)(?:in|for|to)\s+`?([a-zA-Z0-9_-]+)/").ok()?,
            // Match specific component or module mentions
            Regex::new(r"([A-Z][a-zA-Z0-9]+)(?:Component|Service|Module)").ok()?,
        ];

        for pattern in scope_patterns.iter() {
            if let Some(captures) = pattern.captures(&self.raw_message) {
                if let Some(scope) = captures.get(1) {
                    return Some(scope.as_str().to_lowercase());
                }
            }
        }

        None
    }

    fn format_description(&self) -> String {
        let mut description = self.raw_message.trim().to_string();
        
        // Remove common prefixes that might have been added by the LLM
        let prefixes_to_remove = [
            "This commit ",
            "Updates to ",
            "Changes to ",
            "Implements ",
            "Fixes ",
        ];

        for prefix in prefixes_to_remove.iter() {
            if description.to_lowercase().starts_with(&prefix.to_lowercase()) {
                description = description[prefix.len()..].to_string();
            }
        }

        // Ensure the first letter is capitalized
        if let Some(first_char) = description.chars().next() {
            description = format!(
                "{}{}",
                first_char.to_uppercase(),
                &description[first_char.len_utf8()..]
            );
        }

        // Remove any trailing punctuation
        if description.ends_with('.') || description.ends_with('!') {
            description.pop();
        }

        description
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_type() {
        let cases = vec![
            ("Add new feature", "feat"),
            ("Fix bug in login", "fix"),
            ("Update documentation", "docs"),
            ("Format code", "style"),
            ("Refactor user service", "refactor"),
            ("Optimize database queries", "perf"),
            ("Add unit tests", "test"),
            ("Update dependencies", "build"),
            ("Update CI pipeline", "ci"),
            ("Revert last commit", "revert"),
            ("Random change", "chore"),
        ];

        for (message, expected_type) in cases {
            let formatter = CommitFormatter::new(message.to_string());
            assert_eq!(formatter.determine_type(), expected_type);
        }
    }

    #[test]
    fn test_scope_extraction() {
        let formatter = CommitFormatter::new(
            "Update the authentication module with new features".to_string(),
        );
        assert_eq!(formatter.extract_scope(), Some("authentication".to_string()));
    }
}
