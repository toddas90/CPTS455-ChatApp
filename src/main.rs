use std::env;

use clap::{arg, command, Parser};

pub mod feed;
pub mod message;
pub mod user;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    #[arg(short, long)]
    server_address: Option<String>,

    #[arg(short, long)]
    username: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.server_address.is_none() {
        server(args).await;
    } else {
        client(args).await;
    }
}

async fn server(args: Args) {
    // ...
}

async fn client(args: Args) {
    // ...
}
