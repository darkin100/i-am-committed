#[derive(Debug, Clone)]
pub struct CommitType {
    message: String,
}

impl CommitType {
    pub fn new(message: String) -> Self {
        CommitType { message }
    }

    pub fn to_string(&self) -> String {
        self.message.clone()
    }
}
