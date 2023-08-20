# chatrooms-rust
A learning project involving the Rust Programming Language
Inspired by this [video](https://www.youtube.com/watch?v=E8cM12jRH7k)

### About
A server and client pass messages to eachother using
```rust
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Frame {
    Join { username: String, room: u32 },
    Message { username: String, body: String },
    Leave { username: String }
}
```

- Async runtime provided by [Tokio](https://tokio.rs/) and used broadcast channels to function as rooms
between two async tasks
- Used [serde](https://serde.rs/) to encode and decode Message Frames in the server and the client
- Sockets and Channels provided by Rust and Tokio are fantastic
- Transferred Frames across TCP connections as raw data without using WebSockets \([tokio-tungstenite](https://github.com/snapview/tokio-tungstenite)\)

### Usage
Run the server having installed cargo using
```sh
cargo run --bin server
```

Run clients using 
```sh
cargo run --bin client
```
Follow the instructions and send messages back and forth across multiple clients
> **Note**
> Room numbers are integers
