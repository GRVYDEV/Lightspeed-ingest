#[macro_use]
extern crate clap;
extern crate log;
use clap::App;
use env_logger::Env;
use log::info;

mod connection;
mod ftl_codec;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: allow ENV_VARS to define this
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let default_bind_address = "0.0.0.0";
    // update cli.yml to add more flags
    let cli_cfg = load_yaml!("cli.yml");
    let matches = App::from_yaml(cli_cfg).get_matches();

    // Find an address and port to bind to. The search order is as follows:
    // 1.) command line argument
    // 2.) environment variable (LS_INGEST_ADDR)
    // 3.) Default to 0.0.0.0
    let bind_address: &str = match matches.value_of("address") {
        Some(addr) => {
            if addr.is_empty() {
                default_bind_address
            } else {
                addr
            }
        }
        None => default_bind_address,
    };

    let _ = connection::read_stream_key(true);
    info!("Listening on {}:8084", bind_address);
    let listener = TcpListener::bind(format!("{}:8084", bind_address)).await?;

    loop {
        // Wait until someone tries to connect then handle the connection in a new task
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            connection::Connection::init(socket);
            // handle_connection(socket).await;
        });
    }
}
