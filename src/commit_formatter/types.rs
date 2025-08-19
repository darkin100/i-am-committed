use std::fmt;

#[derive(Debug, Clone)]
pub struct CommitType {
    message: String,
}

impl CommitType {
    pub fn new(message: String) -> Self {
        CommitType { message }
    }
}

impl fmt::Display for CommitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
