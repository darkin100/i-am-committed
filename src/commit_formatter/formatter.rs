use super::types::CommitType;

pub struct CommitFormatter {
    raw_message: String,
}

impl CommitFormatter {
    pub fn new(raw_message: String) -> Self {
        CommitFormatter { raw_message }
    }

    pub fn format(&self) -> CommitType {
        // Remove backticks and trim whitespace
        let cleaned_message = self.raw_message
            .replace("`", "")
            .trim()
            .to_string();
        
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
        assert_eq!(formatter.format().to_string(), "feat(auth): Add login functionality");
    }

    #[test]
    fn test_format_message_with_single_backticks() {
        let message = "`feat(core)`: Add new feature";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(formatter.format().to_string(), "feat(core): Add new feature");
    }

    #[test]
    fn test_format_message_with_multiline_backticks() {
        let message = "feat(core): Add new feature
        ```
        added new feature
        ```";
        let formatter = CommitFormatter::new(message.to_string());
        assert_eq!(formatter.format().to_string(), "feat(core): Add new feature
        
        added new feature");
    }
}
