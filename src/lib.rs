use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Frame {
    Join { username: String },
    Message { username: String, body: String },
}
