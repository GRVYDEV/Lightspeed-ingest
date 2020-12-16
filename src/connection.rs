use bytes::{Buf, BufMut, BytesMut};

use hex::{decode, encode};
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use ring::hmac;

use crate::ftl_codec::{FtlCodec, FtlCommand};
use futures::{SinkExt, StreamExt};
use std::str;
use tokio::net::TcpStream;

use tokio::sync::mpsc;
use tokio_util::codec::Framed;

#[derive(Debug)]
enum FrameCommand {
    Send { data: Vec<String> },
    Kill,
}

pub struct Connection {}
pub struct ConnectionState {
    pub hmac_payload: Option<String>,
}

impl ConnectionState {
    pub fn get_payload(&self) -> String {
        match &self.hmac_payload {
            Some(payload) => return payload.clone(),
            None => return "".to_string(),
        }
    }
}
impl Connection {
    pub fn init(stream: TcpStream) {
        let (mut frame_send, mut conn_receive) = mpsc::channel::<FtlCommand>(2);
        let (mut conn_send, mut frame_receive) = mpsc::channel::<FrameCommand>(2);

        tokio::spawn(async move {
            let mut frame = Framed::new(stream, FtlCodec::new());
            loop {
                match frame.next().await {
                    Some(Ok(command)) => {
                        println!("Command was {:?}", command);
                        match frame_send.send(command).await {
                            Ok(_) => {
                                let command = frame_receive.recv().await;
                                match handle_frame_command(command, &mut frame).await {
                                    Ok(_) => {
                                        return;
                                    }
                                    Err(e) => {
                                        println!(
                                            "There was an error handing frame command {:?}",
                                            e
                                        );
                                        return;
                                    }
                                };
                            }
                            _ => {
                                println!(
                                    "There was an error sending the command to the connection"
                                );
                                return;
                            }
                        };
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
        });

        tokio::spawn(async move {
            let mut state = ConnectionState { hmac_payload: None };
            loop {
                match conn_receive.recv().await {
                    Some(command) => {
                        handle_command(command, &conn_send, &mut state).await;
                    }
                    None => {
                        println!("Nothing received from the frame");
                        return;
                    }
                }
            }
        });
    }
}

async fn handle_frame_command(
    command: Option<FrameCommand>,
    frame: &mut Framed<TcpStream, FtlCodec>,
) -> Result<(), String> {
    match command {
        Some(FrameCommand::Send { data }) => {
            let mut d: Vec<String> = data.clone();
            d.reverse();
            while d.len() != 0 {
                match frame.send(d.pop().unwrap()).await {
                    Ok(_) => {}
                    Err(e) => {
                        println!("There was an error {:?}", e);
                        return Err(format!("There was an error {:?}", e));
                    }
                }
            }

            return Ok(());
        }
        Some(FrameCommand::Kill) => {
            println!("TODO: Implement Kill command");
            return Ok(());
        }
        None => {
            println!("Error receiving command from conn");
            return Err("Error receiving command from conn".to_string());
        }
    };
}

async fn handle_command(
    command: FtlCommand,
    sender: &mpsc::Sender<FrameCommand>,
    conn: &mut ConnectionState,
) {
    let mut resp: Vec<String>;
    match command {
        FtlCommand::HMAC => {
            resp = Vec::new();
            println!("Handling HMAC Command");
            conn.hmac_payload = Some(generate_hmac());
            resp.push("200 ".to_string());
            resp.push(conn.get_payload());
            resp.push("\n".to_string());
            match sender.send(FrameCommand::Send { data: resp }).await {
                Ok(_) => {
                    return;
                }
                Err(e) => {
                    println!("Error sending to frame task (From: Handle HMAC) {:?}", e);
                    return;
                }
            }
        }
        FtlCommand::Connect { data } => {
            resp = Vec::new();
            println!("Handling Connect Command");
            match (data.get("stream_key"), data.get("channel_id")) {
                (Some(key), Some(_channel_id)) => {
                    let client_hash = hex::decode(key).expect("error with hash decode");
                    //TODO: Add a more elegant stream key system
                    let key =
                        hmac::Key::new(hmac::HMAC_SHA512, b"aBcDeFgHiJkLmNoPqRsTuVwXyZ123456");
                    match hmac::verify(
                        &key,
                        decode(conn.get_payload().into_bytes())
                            .expect("error with payload decode")
                            .as_slice(),
                        &client_hash.as_slice(),
                    ) {
                        Ok(_) => {
                            println!("Hashes equal!");
                            resp.push("200\n".to_string());
                            match sender.send(FrameCommand::Send { data: resp }).await {
                                Ok(_) => {
                                    return;
                                }
                                Err(e) => println!(
                                    "Error sending to frame task (From: Handle Connection) {:?}",
                                    e
                                ),
                            }
                        }
                        _ => {
                            println!("Hashes do not equal");
                            return;
                        }
                    };
                    //temp stream key aBcDeFgHiJkLmNoPqRsTuVwXyZ123456
                }

                (None, _) => {
                    println!("No stream key attached to connect command");
                    return;
                }
                (_, None) => {
                    println!("No channel id attached to connect command");
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
