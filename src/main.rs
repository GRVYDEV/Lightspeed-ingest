use tokio::io;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = TcpListener::bind("0.0.0.0:8084").await?;

    loop {
        let (socket, _) = listener.accept().await?;

        handle_connection(socket).await;
    }
}

async fn handle_connection(mut stream: TcpStream) {
    tokio::spawn(async move {
        // In a loop, read data from the socket and write the data back.

        println!("Sender addr: {:?}", stream.peer_addr());
        let mut buffer = [0; 1024];

        match stream.read(&mut buffer).await {
            Ok(var) => println!("Success {:?}", String::from_utf8_lossy(&buffer[..]),
            Err(err) => println!("there was an error {:?}", err),
        };
    });
}
