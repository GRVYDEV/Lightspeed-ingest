#[macro_use]
extern crate clap;
use clap::App;

mod connection;
mod ftl_codec;
use tokio::net::TcpListener;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let default_bind_address = "0.0.0.0";
    let default_port = "8084";

    // update cli.yml to add more flags
    let cli_cfg = load_yaml!("cli.yml");
    let matches = App::from_yaml(cli_cfg).get_matches();

    // Find an address and port to bind to. The search order is as follows:
    // 1.) command line argument
    // 2.) environment variables (INGEST_ADDR and INGEST_PORT)
    // 3.) Default to 0.0.0.0 and 8084
    let bind_address: &str = match matches.value_of("address") {
        Some(addr) => {
            if addr.is_empty() {
                default_bind_address
            }
            else {
                addr
            }
        }
        None => default_bind_address
    };

    let port: &str = match matches.value_of("port") {
        Some(port) => {
            if port.is_empty() {
                default_port
            }
            else {
                port
            }
        }
        None => default_port
    };

    println!("Listening on {} port {}", bind_address, port);
    let listener = TcpListener::bind(format!("{}:{}", bind_address, port)).await?;

    loop {
        // Wait until someone tries to connect then handle the connection in a new task
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            connection::Connection::init(socket);
            // handle_connection(socket).await;
        });
    }
}
