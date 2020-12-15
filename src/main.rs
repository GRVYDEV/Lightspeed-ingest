mod ftl_codec;
use bytes::{Buf, BufMut, BytesMut};
use ftl_codec::*;
use futures::stream::TryStreamExt;
use futures::{stream, SinkExt, StreamExt};
use hex::encode;
use hmac::{Hmac, Mac, NewMac};
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use sha2::Sha512;
use std::str;
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
    println!("Sender addr: {:?}", stream.peer_addr().unwrap());
    let mut frame = Framed::new(stream, FtlCodec::new());
    loop {
        match frame.next().await {
            Some(Ok(command)) => {
                // println!("Command was {:?}", command);
                handle_command(command, &mut frame).await;
            }
            Some(Err(e)) => {
                println!("There was an error {:?}", e);
                return;
            }
            None => {
                println!("There was a socket reading error");
                return;
            }
        };
    }
}

async fn handle_command(command: FtlCommand, frame: &mut Framed<TcpStream, FtlCodec>) {
    match command.command {
        Command::HMAC => {
            println!("Handling HMAC Command");
            frame.codec_mut().set_hmac(generate_hmac());
            println!(
                "payload generated {:?}",
                frame.codec().hmac_payload.clone().unwrap()
            );
            let mut resp: Vec<String> = Vec::new();
            resp.push("200 ".to_string());
            resp.push(frame.codec().hmac_payload.clone().unwrap());
            resp.push("\n".to_string());
            match frame.send(&mut resp.get_mut(0).unwrap()).await {
                Ok(_) => {}
                Err(e) => {
                    println!("There was an error {:?}", e);
                    return;
                }
            };
            match frame.send(&mut resp.get_mut(1).unwrap()).await {
                Ok(_) => {}
                Err(e) => {
                    println!("There was an error {:?}", e);
                    return;
                }
            };
            match frame.send(&mut resp.get_mut(2).unwrap()).await {
                Ok(_) => {
                    return;
                }
                Err(e) => {
                    println!("There was an error {:?}", e);
                    return;
                }
            }
        }
        Command::Connect => {
            println!("Handling Connect Command");
            match command.data {
                Some(data) => {
                    println!("channel id: {:?}", data.get(&"channel_id".to_string()));
                    type HmacSha512 = Hmac<Sha512>;

                    let mut mac = HmacSha512::new_varkey(
                        frame
                            .codec_mut()
                            .hmac_payload
                            .clone()
                            .unwrap()
                            .into_bytes()
                            .as_slice(),
                    )
                    .expect("some err");
                    mac.update(b"aBcDeFgHiJkLmNoPqRsTuVwXyZ123456");
                    let res = mac.finalize().into_bytes();
                    let result = str::from_utf8(&res);
                    println!(
                        "client hash: {:?}",
                        data.get(&"stream_key".to_string()).unwrap()
                    );
                    println!("server hash: {:?}", result);
                    //temp stream key aBcDeFgHiJkLmNoPqRsTuVwXyZ123456
                    return;
                }

                None => {
                    println!("No data attached to connect command");
                    return;
                }
            }
        }
        _ => {
            println!("Command not implemented yet. Tell GRVY to quit his day job");
            return;
        }
    }
}

fn generate_hmac() -> String {
    let dist = Uniform::new(0x00, 0xFF);
    let mut hmac_payload: Vec<u8> = Vec::new();
    let mut rng = thread_rng();
    for _i in 0..128 {
        hmac_payload.push(rng.sample(dist));
    }
    encode(hmac_payload.as_slice())
}
