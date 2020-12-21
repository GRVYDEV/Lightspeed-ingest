use bytes::BytesMut;
use rtp_rs::*;
use std::io;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::sync::broadcast::Receiver;
use tokio::sync::{broadcast, mpsc, oneshot};

#[derive(Debug, Clone)]
pub enum UdpRelayCommand {
    Send { data: Vec<u8> },
    Kill,
}

pub enum UdpMessage {
    GetUniqueId { respond_to: oneshot::Sender<u32> },
}

pub async fn receive_start(
    recv_socket_port: String,
    relay_send: broadcast::Sender<UdpRelayCommand>,
) {
    tokio::select! {
        output = real_receive_start(recv_socket_port, relay_send) => output,
    }
}

pub async fn real_receive_start(
    recv_socket_port: String,
    relay_send: broadcast::Sender<UdpRelayCommand>,
) {
    let recv_socket = UdpSocket::bind("10.17.0.5:65535")
        .await
        .expect("Failed to bind to port");
    let mut bytes = BytesMut::with_capacity(2000);
    
    loop {
        // let mut buf = [0 as u8; 2000];
        match recv_socket.recv(&mut bytes).await {
            Ok(n) => {
                println!("Receieved {:?} bytes", n);
                if let Ok(rtp) = RtpReader::new(&bytes.to_vec()) {
                    println!("Receieved {:?}", rtp);
                };
                match relay_send.send(UdpRelayCommand::Send {
                    data: bytes.to_vec(),
                }) {
                    Ok(_) => {
                        bytes.clear();
                    }
                    Err(e) => {
                        println!(
                            "There was an error sending the packet to the relay task. Error: {:?}",
                            e
                        )
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
}

pub async fn relay_start(relay_receive: broadcast::Sender<UdpRelayCommand>) {
    tokio::select! {
        output = real_relay_start(relay_receive) => output,

    }
}

pub async fn real_relay_start(relay_receive: broadcast::Sender<UdpRelayCommand>) {
    let send_socket = UdpSocket::bind("127.0.0.1:9000")
        .await
        .expect("failed to bind to 127.0.0.1:9000");
    let mut recv: broadcast::Receiver<UdpRelayCommand> = relay_receive.subscribe();
    match send_socket.connect("127.0.0.1:9000").await {
        Ok(_) => {}
        Err(e) => println!("There was an error connecting to udp socket {:?}", e),
    }
    loop {
        match recv.recv().await {
            Ok(UdpRelayCommand::Send { data }) => {
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
            Ok(UdpRelayCommand::Kill) => {}
            Err(_) => {}
        }
    }
}
pub fn init(recv_socket_port: String) {
    let (relay_send, mut relay_receive) = mpsc::channel::<UdpRelayCommand>(2);
    tokio::spawn(async move {
        let recv_socket = UdpSocket::bind("10.17.0.5:65535")
            .await
            .expect("Failed to bind to port");
        let mut bytes = BytesMut::with_capacity(2000);
        loop {
            match recv_socket.recv(&mut bytes).await {
                Ok(n) => {
                    if let Ok(rtp) = RtpReader::new(&bytes) {
                        println!("Receieved {:?}", rtp);
                    };
                    match relay_send
                        .send(UdpRelayCommand::Send {
                            data: bytes.to_vec(),
                        })
                        .await
                    {
                        Ok(_) => {
                            bytes.clear();
                        }
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
