use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Frame {
    Join { username: String, room: u32 },
    Message { username: String, body: String },
    Leave { username: String }
}
