mod connection;
mod ftl_codec;
mod rtp_relay;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Listening on port 8084");
    let listener = TcpListener::bind("0.0.0.0:8084").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            connection::Connection::init(socket);
            // handle_connection(socket).await;
        });
    }
}
