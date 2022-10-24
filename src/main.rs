use std::error::Error;

use clap::{arg, command, Parser};

pub mod feed;
pub mod message;
pub mod user;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // If the port is not specified, we'll use 8080.
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    // If the host is not specified, we'll assume this user wants to create the server.
    #[arg(short, long)]
    server_address: Option<String>,

    // If the username is not specified, we'll use "Anonymous".
    #[arg(short, long)]
    username: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.server_address.is_none() {
        server(args).await;
    } else {
        client(args).await;
    }
    Ok(())
}

async fn server(args: Args) {
    let addr = format!("127.0.0.1:{}", args.port);
}

async fn client(args: Args) {
    // ...
}
