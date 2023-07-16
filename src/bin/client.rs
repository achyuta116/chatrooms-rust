use tokio::{
    self,
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use chat_app::Frame;

#[tokio::main]
async fn main() -> io::Result<()> {
    let client = TcpStream::connect("127.0.0.1:5300").await.unwrap();
    let (mut rd, mut wr) = client.into_split();
    let data = serde_json::to_string(&Frame::Join {
        username: "amazinggg".to_string(),
    })
    .unwrap();

    let data = data.as_bytes();
    wr.write_u32(data.len().try_into().unwrap()).await.unwrap();
    wr.write_all(data).await.unwrap();

    let stdin = std::io::stdin();
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
                                println!("{username}: {body}");
                            },
                            _ => {
                                eprintln!("Undefined frame");
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read from socket: {e}");
                    break;
                }
            }
        }
    });
    loop {
        let mut message = String::new();
        if let Err(e) = stdin.read_line(&mut message) {
            eprintln!("Stdin closed!: {e}");
            break;
        }

        let data = serde_json::to_string(&Frame::Message {
            username: "amazinggg".to_string(),
            body: message,
        })
        .unwrap();
        let data = data.as_bytes();

        wr.write_u32(data.len().try_into().unwrap()).await.unwrap();
        wr.write_all(data).await.unwrap();
    }
    Ok(())
}

