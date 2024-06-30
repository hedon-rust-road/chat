mod chat;
mod file;
mod message;
mod user;
mod workspace;

pub use chat::*;
pub use message::*;
use serde::{Deserialize, Serialize};
pub use user::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatFile {
    pub ws_id: u64,
    pub ext: String, // extract ext from filename or mime type
    pub hash: String,
}
