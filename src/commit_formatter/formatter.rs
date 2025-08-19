use super::types::CommitType;

pub struct CommitFormatter {
    raw_message: String,
}

impl CommitFormatter {
    pub fn new(raw_message: String) -> Self {
        CommitFormatter { raw_message }
    }

    pub fn format(&self) -> CommitType {
        // Remove backticks, XML commit_message tags, and trim whitespace
        let mut cleaned_message = self.raw_message.clone();
        
        // Remove commit_message XML tags
        cleaned_message = cleaned_message
            .replace("<commit_message>", "")
            .replace("</commit_message>", "");
        
        // Remove backticks
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
        assert_eq!(formatter.format().to_string(), message);
    }

    #[test]
    fn test_format_message_with_backticks() {
        let message = "```feat(auth): Add login functionality```";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            formatter.format().to_string(),
            "feat(auth): Add login functionality"
        );
    }

    #[test]
    fn test_format_message_with_single_backticks() {
        let message = "`feat(core)`: Add new feature";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            formatter.format().to_string(),
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
            formatter.format().to_string(),
            "feat(core): Add new feature
        
        added new feature"
        );
    }

    #[test]
    fn test_format_message_with_commit_message_tags() {
        let message = "<commit_message>feat(auth): Add login functionality</commit_message>";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            formatter.format().to_string(),
            "feat(auth): Add login functionality"
        );
    }

    #[test]
    fn test_format_message_with_commit_message_tags_and_backticks() {
        let message = "<commit_message>`feat(core)`: Add new feature</commit_message>";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(
            formatter.format().to_string(),
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
            formatter.format().to_string(),
            "feat(core): Add new feature

Added a new feature to the core module"
        );
    }
}
