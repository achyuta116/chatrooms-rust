use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};

use chat_app::Frame;

#[tokio::main()]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5300").await?;
    let rooms = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (socket, _) = listener.accept().await?;
        let rooms_shared = Arc::clone(&rooms);
        tokio::spawn(async move { process_client(socket, rooms_shared).await });
    }
}

async fn process_client(mut socket: TcpStream, rooms: Arc<Mutex<HashMap<u32, Sender<Frame>>>>) {
    let room_number;
    match socket.read_u32().await {
        Ok(0) => {
            eprintln!("Socket did not send a join message!");
            return;
        }
        Ok(size) => {
            let mut data = vec![0; size.try_into().unwrap()];
            if let Err(e) = socket.read_exact(&mut data).await {
                eprintln!("Failed to read from socket: {e}");
                return;
            };

            if let Ok(frame) = serde_json::from_slice(&data) {
                match frame {
                    Frame::Join { username, room } => {
                        let mut rooms = rooms.lock().unwrap();
                        if rooms.get(&room).is_some() {
                            room_number = room;
                        } else {
                            let (tx, _) = broadcast::channel(32);
                            rooms.insert(room, tx);
                            room_number = room;
                        }
                        println!("{username} joined the chat {room_number}!");
                    }
                    _ => {
                        eprintln!("Socket did not send a join message!");
                        return;
                    }
                }
            } else {
                eprintln!("Failed to parse message from socket!");
                return;
            }
        }
        Err(e) => {
            eprintln!("Failure to read from socket! {e}");
            return;
        }
    };

    let tx = {
        let rooms = rooms.lock().unwrap();
        let tx = rooms.get(&room_number).unwrap();
        tx.clone()
    };

    let mut rx = tx.subscribe();
    let (mut rd, mut wr) = socket.into_split();
    tokio::spawn(async move {
        while let Ok(frame) = rx.recv().await {
            match frame {
                Frame::Message { username, body } => {
                    let data = serde_json::to_string(&Frame::Message { username, body }).unwrap();
                    let data = data.as_bytes();

                    if let Err(e) = wr.write_u32(data.len().try_into().unwrap()).await {
                        eprintln!("Failed to write to socket: {e}");
                        break;
                    }

                    if let Err(e) = wr.write_all(data).await {
                        eprintln!("Failed to write to socket: {e}");
                        break;
                    }
                }

                Frame::Leave { username } => {
                    let data = serde_json::to_string(&Frame::Leave { username }).unwrap();
                    let data = data.as_bytes();

                    if let Err(e) = wr.write_u32(data.len().try_into().unwrap()).await {
                        eprintln!("Failed to write to socket: {e}");
                        break;
                    }

                    if let Err(e) = wr.write_all(data).await {
                        eprintln!("Failed to write to socket: {e}");
                        break;
                    }
                }
                _ => todo!(),
            };
        }
    });

    tokio::spawn(async move {
        loop {
            match rd.read_u32().await {
                Ok(0) => break,
                Ok(size) => {
                    let mut data = vec![0; size.try_into().unwrap()];
                    if let Err(e) = rd.read_exact(&mut data).await {
                        eprintln!("Failed to read from socket: {e}");
                        break;
                    }

                    if let Ok(frame) = serde_json::from_slice(&data) {
                        match frame {
                            Frame::Message { username, body } => {
                                println!("{username} left message: {body} in {room_number}");
                                tx.send(Frame::Message { username, body }).unwrap();
                            }
                            Frame::Leave { username } => {
                                println!("{username} left the chat {room_number}!");
                                if tx.receiver_count() > 1 {
                                    tx.send(Frame::Leave { username }).unwrap();
                                } else {
                                    let mut rooms = rooms.lock().unwrap();
                                    rooms.retain(|_, tx| tx.receiver_count() > 1); 
                                }
                                break;
                            }
                            _ => {
                                eprintln!("Received incorrect frame!");
                                break;
                            }
                        };
                    } else {
                        eprintln!("Failed to parse frame from socket");
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read from socket: {e}");
                    break;
                }
            }
        }
    });
}

