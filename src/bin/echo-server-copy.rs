use tokio::net::TcpListener;
use tokio::io;

#[tokio::main()]
async fn main() -> tokio::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await.unwrap();
    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let (mut rd, mut wr ) = io::split(socket);
            if io::copy(&mut rd, &mut wr).await.is_err() {
                eprintln!("failed to copy");
            }
        });
    };
}
