mod ftl_codec;
use ftl_codec::*;

use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio_util::codec::{Decoder, Encoder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Listening on port 8084");
    let mut listener = TcpListener::bind("0.0.0.0:8084").await?;

    loop {
        let (socket, _) = listener.accept().await?;

        handle_connection(socket).await;
    }
}

async fn handle_connection(mut stream: TcpStream) {
    tokio::spawn(async move {
        // In a loop, read data from the socket and write the data back.

        println!("Sender addr: {:?}", stream.peer_addr().unwrap());
        let mut buffer = bytes::BytesMut::with_capacity(1024);

        match stream.read(&mut buffer).await {
            Ok(var) => {println!("bytes read {:?}", var); handle_message(&mut buffer, var)},
            Err(err) => println!("There was a socket reading error {:?}", err),
        };
    });
}

fn handle_message(message: &mut bytes::BytesMut, bytes_read: usize) {
    let mut ftl_codec = FtlCodec::new(bytes_read);
    let command = ftl_codec.decode(message);
    println!("Command returned from codec: {:?}", command);
}
