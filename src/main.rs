use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::UdpSocket;
use std::thread;
fn main() {
    let listener = TcpListener::bind("10.10.0.5:8084").unwrap();
    println!("Listening on port 8084");
    for stream in listener.incoming() {
        println!("Connected");

        thread::spawn(|| match stream {
            Ok(stream) => handle_connection(stream),
            Err(Error) => println!("Error"),
        });
        // match stream{
        //     Ok((socket, addr)) => {
        //         println!("new client: {:?} stream {:?}", addr, socket);
        //         handle_connection(socket)
        //     }
        //     Err(e) => println!("couldn't get client: {:?}", e),
        // }
    }

    // let socket = UdpSocket::bind("127.0.0.1:8084").expect("couldn't bind to address");
    // println!("UDP Socket on 8084");
    // let mut buf = [0; 10];
    // let (number_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("Didn't receive data");

    // println!("{:?}", src_addr);
}

fn handle_connection(mut stream: TcpStream) {
    println!("Sender addr: {:?}", stream.peer_addr());
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}
