use super::types::CommitType;

pub struct CommitFormatter {
    raw_message: String,
}

impl CommitFormatter {
    pub fn new(raw_message: String) -> Self {
        CommitFormatter { raw_message }
    }

    pub fn format(&self) -> CommitType {
        CommitType::new(self.raw_message.trim().to_string())
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
}
