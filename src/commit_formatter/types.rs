pub const COMMIT_TYPES: &[&str] = &[
    "feat", "fix", "chore", "docs", "style",
    "refactor", "perf", "test", "build", "ci", "revert"
];

#[derive(Debug, Clone)]
pub struct CommitType {
    pub type_str: String,
    pub scope: Option<String>,
    pub description: String,
    pub breaking_change: bool,
}

impl CommitType {
    pub fn new(
        type_str: String,
        scope: Option<String>,
        description: String,
        breaking_change: bool,
    ) -> Self {
        CommitType {
            type_str,
            scope,
            description,
            breaking_change,
        }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        
        // Add type and scope
        result.push_str(&self.type_str);
        if let Some(scope) = &self.scope {
            result.push_str(&format!("({})", scope));
        }
        
        // Add description
        result.push_str(": ");
        result.push_str(&self.description);
        
        // Add breaking change footer if needed
        if self.breaking_change {
            result.push_str("\n\nBREAKING CHANGE: This commit introduces breaking changes");
        }
        
        result
    }
}
