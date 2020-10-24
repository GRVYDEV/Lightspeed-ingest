mod ftl_codec;
use bytes::{Buf, BufMut, BytesMut};
use ftl_codec::*;
use futures::StreamExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio_util::codec::{Decoder, Encoder, Framed};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Listening on port 8084");
    let mut listener = TcpListener::bind("0.0.0.0:8084").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }
}

async fn handle_connection(mut stream: TcpStream) {
    // In a loop, read data from the socket and write the data back.
    println!("Sender addr: {:?}", stream.peer_addr().unwrap());
    let mut frame = Framed::new(stream, FtlCodec::new());
    loop {
        match frame.next().await {
            Some(result) => match result {
                Ok(command) => {
                    println!("Command was {:?}", command);
                    handle_command(command, &mut frame);
                    return;
                }
                Err(e) => {
                    println!("There was an error: {:?}", e);
                    return;
                }
            },
            None => {
                println!("There was a socket reading error");
                return;
            }
        };
    }
}

fn handle_command(command: FtlCommand, frame: &mut Framed<TcpStream, FtlCodec>) {
    match command.command {
        Command::HMAC => println!("Handling HMAC Command"),
        _ => println!("Command not implemented yet. Tell GRVY to quit his day job"),
    }
}

fn generate_hmac() {}
