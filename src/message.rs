use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub user: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.user, self.body)
    }
}

impl Message {
    pub fn new(user: &str, body: &str, created_at: DateTime<Utc>) -> Self {
        Message {
            user: user.to_string(),
            body: String::from(body),
            created_at,
        }
    }
}
