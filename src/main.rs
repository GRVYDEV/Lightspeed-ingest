mod ftl_codec;
use bytes::{Buf, BufMut, BytesMut};
use ftl_codec::*;
use futures::stream::TryStreamExt;
use futures::{stream, SinkExt, StreamExt};
use hex::{decode, encode};
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use ring::hmac;
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
    let mut resp: Vec<String>;
    match command.command {
        Command::HMAC => {
            resp = Vec::new();
            println!("Handling HMAC Command");
            frame.codec_mut().set_hmac(generate_hmac());
            resp.push("200 ".to_string());
            resp.push(frame.codec().hmac_payload.clone().unwrap());
            resp.push("\n".to_string());
            match frame.send("200 ".to_string()).await {
                Ok(_) => {}
                Err(e) => {
                    println!("There was an error {:?}", e);
                    return;
                }
            };
            match frame
                .send(frame.codec().hmac_payload.clone().unwrap())
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    println!("There was an error {:?}", e);
                    return;
                }
            };
            match frame.send("\n".to_string()).await {
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
            resp = Vec::new();
            println!("Handling Connect Command");
            match command.data {
                Some(data) => {
                    let client_hash =
                        hex::decode(data.get(&"stream_key".to_string()).unwrap().clone())
                            .expect("error with hash decode");
                    //TODO: Add a more elegant stream key system
                    let key =
                        hmac::Key::new(hmac::HMAC_SHA512, b"aBcDeFgHiJkLmNoPqRsTuVwXyZ123456");
                    match hmac::verify(
                        &key,
                        decode(
                            frame
                                .codec_mut()
                                .hmac_payload
                                .clone()
                                .unwrap()
                                .into_bytes()
                                .as_slice(),
                        )
                        .expect("error with payload decode")
                        .as_slice(),
                        &client_hash.as_slice(),
                    ) {
                        Ok(_) => {
                            println!("Hashes equal!");
                            resp.push("200\n".to_string());
                            match frame.send("200\n".to_string()).await {
                                Ok(_) => {
                                    return;
                                }
                                Err(e) => {
                                    println!("There was an error {:?}", e);
                                    return;
                                }
                            }
                        }
                        _ => {
                            println!("Hashes do not equal");
                            return;
                        }
                    };
                    // println!("client hash: {:?}", &client_hash);
                    // // println!("are they equal? {:?}", mac.verify(&client_hash));
                    // println!("server hash {:?}", tag.as_ref());
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

use std::cmp;
fn compare(a: &[u8], b: &[u8]) -> cmp::Ordering {
    a.iter()
        .zip(b)
        .map(|(x, y)| x.cmp(y))
        .find(|&ord| ord != cmp::Ordering::Equal)
        .unwrap_or(a.len().cmp(&b.len()))
}
