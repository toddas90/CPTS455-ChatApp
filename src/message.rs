use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::user;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextMessage {
    pub username: String,
    pub user_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMessage {
    pub username: String,
    pub user_id: Uuid,
    pub file_name: String,
    pub file_size: usize,
    pub file_data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub username: String,
    pub user_id: Uuid,
    pub command: String,
    pub args: Vec<String>,
}

impl TextMessage {
    pub fn new(user: &user::User, body: &str, created_at: DateTime<Utc>) -> Self {
        TextMessage {
            username: user.username.to_owned(),
            user_id: user.user_id,
            body: String::from(body),
            created_at,
        }
    }
}

impl FileMessage {
    pub fn new(user: &user::User, file_name: &str, file_size: usize, file_data: &Vec<u8>) -> Self {
        FileMessage {
            username: user.username.to_owned(),
            user_id: user.user_id,
            file_name: String::from(file_name),
            file_size,
            file_data: file_data.to_owned(),
        }
    }
}

impl Command {
    pub fn new(user: &user::User, command: &str, args: Vec<&str>) -> Self {
        Command {
            username: user.username.to_owned(),
            user_id: user.user_id,
            command: String::from(command),
            args: args.iter().map(|s| String::from(*s)).collect(),
        }
    }
}

pub fn encrypt(message: String) -> String {
    // Simple encryption using caesar cipher with 7 as the key
    let mut encrypted = String::new();
    for c in message.chars() {
        let mut new_c = c as u8 + 7;
        if new_c > 126 {
            new_c -= 95;
        }
        encrypted.push(new_c as char);
    }
    encrypted
}

pub fn decrypt(cipher: &[u8]) -> String {
    // Simple decryption using caesar cipher with 7 as the key
    let mut decrypted = String::new();
    for c in cipher {
        let mut new_c = *c as u8 - 7;
        if new_c < 32 {
            new_c += 95;
        }
        decrypted.push(new_c as char);
    }
    decrypted
}
