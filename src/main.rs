use tokio::io;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::prelude::*;

const COMMAND_DELIMITERS: [char; 4] = ['\r', '\n', '\r', '\n'];
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
        let mut buffer = [0; 1024];

        match stream.read(&mut buffer).await {
            Ok(var) => handle_message(&buffer, &var),
            Err(err) => println!("There was a socket reading error {:?}", err),
        };
    });
}

fn handle_message(message: &[u8], bytes_read: &usize) {
    let close_size: usize = 0;
    let mut command_buffer: std::vec::Vec<u8> = Vec::new();
    if bytes_read == &close_size {
        println!("Connection Closed");
    } else {
        let mut delimiter_characters: usize = 0;
        for i in 0..message.len() {
            command_buffer.push(message[i]);
            if message[i] as char == COMMAND_DELIMITERS[delimiter_characters] {
                delimiter_characters += 1;
                if delimiter_characters >= COMMAND_DELIMITERS.len() {
                    let command_slice: &[u8] = command_buffer.as_slice();
                    let mut command = String::from_utf8_lossy(&command_slice).to_string();
                    command.truncate(command.len() - 4);
                    println!("Command: {:?}", command);
                }
            }
        }
    }
}
