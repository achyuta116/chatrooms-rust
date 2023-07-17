use tokio::{
    self,
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use chat_app::Frame;

#[tokio::main]
async fn main() -> io::Result<()> {
    let client = TcpStream::connect("127.0.0.1:5300").await.unwrap();
    // Read name from stdin
    let mut username = String::new();
    println!("Enter your username:");
    std::io::stdin()
        .read_line(&mut username)
        .expect("Failed to read line");
    let username = username.trim().to_string();

    // Read room number from stdin
    let mut room_number = String::new();
    println!("Enter your room number:");
    std::io::stdin()
        .read_line(&mut room_number)
        .expect("Failed to read line");
    let room_number: u32 = room_number.trim().parse().expect("Invalid room number");

    let (mut rd, mut wr) = client.into_split();
    let data = serde_json::to_string(&Frame::Join {
        username: String::from(&username),
        room: room_number,
    })
    .unwrap();

    let data = data.as_bytes();
    wr.write_u32(data.len().try_into().unwrap()).await.unwrap();
    wr.write_all(data).await.unwrap();

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
                            }
                            Frame::Leave { username } => {
                                println!("{username} left the chat!");
                            }
                            _ => {
                                eprintln!("Received incorrect frame!");
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

    let stdin = std::io::stdin();
    loop {
        let mut message = String::new();
        if let Err(e) = stdin.read_line(&mut message) {
            eprintln!("Stdin closed!: {e}");
            break;
        }
        let message = message.trim();

        if message == String::from("quit") {
            let data = serde_json::to_string(&Frame::Leave {
                username: String::from(&username),
            })
            .unwrap();
            let data = data.as_bytes();

            wr.write_u32(data.len().try_into().unwrap()).await.unwrap();
            wr.write_all(data).await.unwrap();
            break;
        }

        let data = serde_json::to_string(&Frame::Message {
            username: String::from(&username),
            body: message.to_string(),
        })
        .unwrap();
        let data = data.as_bytes();

        wr.write_u32(data.len().try_into().unwrap()).await.unwrap();
        wr.write_all(data).await.unwrap();
    }
    Ok(())
}

