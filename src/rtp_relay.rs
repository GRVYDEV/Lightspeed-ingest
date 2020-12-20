use crate::ftl_codec::{FtlCodec, FtlCommand};
use futures::{SinkExt, StreamExt};
use hex::{decode, encode};
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use ring::hmac;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio_util::codec::Framed;

pub struct UdpConnection {}

#[derive(Debug)]
pub enum UdpRelayCommand {
    Send { data: Vec<u8> },
    Kill,
}

impl UdpConnection {
    pub fn init(recv_socket_port: String) {
        let (relay_send, mut relay_receive) = mpsc::channel::<UdpRelayCommand>(2);
        tokio::spawn(async move {
            let recv_socket = UdpSocket::bind(format!("0.0.0.0:{}", recv_socket_port))
                .await
                .expect("Failed to bind to port");
                
            loop {
                let mut buf = [0 as u8];
                println!("Connected to udp socket");
                match recv_socket.recv(&mut buf).await {
                    Ok(_) => {
                        println!("Receieved");
                        match relay_send
                            .send(UdpRelayCommand::Send { data: buf.to_vec() })
                            .await
                        {
                            Ok(_) => {}
                            Err(e) => {
                                println!("There was an error sending the packet to the relay task. Error: {:?}", e)
                            }
                        }
                    }
                    Err(e) => {
                        println!("There was an error reading from the socket. Error: {:?}", e)
                    }
                }
            }
        });

        tokio::spawn(async move {
            let send_socket = UdpSocket::bind("127.0.0.1:9000")
                .await
                .expect("failed to bind to 127.0.0.1:9000");

            loop {
                match relay_receive.recv().await {
                    Some(UdpRelayCommand::Send { data }) => {
                        println!("Received UDP RELAY COMMAND");
                        // match send_socket.send(data.as_slice()).await {
                        //     Ok(_) => {}
                        //     Err(e) => {
                        //         println!(
                        //             "Failed to send packet to loopback interface. Error: {:?}",
                        //             e
                        //         )
                        //     }
                        // };
                    }
                    Some(UdpRelayCommand::Kill) => {}
                    None => {}
                }
            }
        });
    }
}
