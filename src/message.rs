use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::user;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub username: String,
    pub user_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.username, self.body)
    }
}

impl Message {
    pub fn new(user: &user::User, body: &str, created_at: DateTime<Utc>) -> Self {
        Message {
            username: user.username.to_owned(),
            user_id: user.user_id,
            body: String::from(body),
            created_at,
        }
    }
}
