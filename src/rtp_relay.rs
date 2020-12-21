use std::net::SocketAddr;
use crate::ftl_codec::{FtlCodec, FtlCommand};
use futures::{SinkExt, StreamExt};
use hex::{decode, encode};
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use ring::hmac;
use std::io;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio_util::codec::BytesCodec;
use tokio_util::codec::Framed;
pub struct UdpConnection {}

#[derive(Debug)]
pub enum UdpRelayCommand {
    Send { data: Vec<u8> },
    Kill,
}

impl UdpConnection {
    pub fn init(recv_socket_port: String, addr: SocketAddr) {
        let (relay_send, mut relay_receive) = mpsc::channel::<UdpRelayCommand>(2);
        tokio::spawn(async move {
            let recv_socket = UdpSocket::bind("10.17.0.5:65535")
                .await
                .expect("Failed to bind to port");

            match recv_socket
                .connect(addr)
                .await
            {
                Ok(_) => {println!("udp connected");}
                Err(e) => println!("There was an error connecting to udp socket {:?}", e),
            };
            
            loop {
                match recv_socket.readable().await {
                    Ok(_) => {println!("Socket is readable")}
                    Err(e) => println!("Error waiting for socket to be readable {:?}", e),
                };
                let mut buf = [0 as u8];

                match recv_socket.try_recv(&mut buf) {
                    Ok(n) => {
                        println!("Receieved {:?} bytes", n);
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
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        println!("would block");
                        continue;
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

            match send_socket.connect("127.0.0.1:9000").await {
                Ok(_) => {}
                Err(e) => println!("There was an error connecting to udp socket {:?}", e),
            }
            loop {
                match relay_receive.recv().await {
                    Some(UdpRelayCommand::Send { data }) => {
                        println!("Received UDP RELAY COMMAND");
                        match send_socket.send(data.as_slice()).await {
                            Ok(_) => {}
                            Err(e) => {
                                println!(
                                    "Failed to send packet to loopback interface. Error: {:?}",
                                    e
                                )
                            }
                        };
                    }
                    Some(UdpRelayCommand::Kill) => {}
                    None => {}
                }
            }
        });
    }
}
