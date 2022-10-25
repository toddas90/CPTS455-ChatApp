use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub username: String,
    pub user_id: Uuid,
}

impl User {
    pub fn new(username: &str) -> Self {
        User {
            username: username.to_string(),
            user_id: Uuid::new_v4(),
        }
    }
}
