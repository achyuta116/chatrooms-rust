use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Receiver, Sender},
};

use chat_app::Frame;

#[tokio::main()]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5300").await?;
    let (tx, rx) = broadcast::channel(32);
    loop {
        let (socket, _) = listener.accept().await?;
        let (rx, tx) = (rx.resubscribe(), tx.clone());
        tokio::spawn(async move { process_client(socket, rx, tx).await });
    }
}

async fn process_client(socket: TcpStream, mut rx: Receiver<Frame>, tx: Sender<Frame>) {
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
                            Frame::Join { username } => {
                                println!("username {username} joined the chat!");
                            }
                            Frame::Message { username, body } => {
                                println!("username: {username} left message: {body}");
                                tx.send(Frame::Message { username, body }).unwrap();
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
