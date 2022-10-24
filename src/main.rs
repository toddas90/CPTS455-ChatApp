use std::error::Error;

use clap::{arg, command, Parser};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
        server(args).await?;
    } else {
        client(args).await;
    }
    Ok(())
}

async fn server(args: Args) -> Result<(), Box<dyn Error>> {
    let addr = format!("127.0.0.1:{}", args.port);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to address {}", addr));

    loop {
        let (mut socket, _) = listener.accept().await?;
        println!("New connection: {}", socket.peer_addr()?);

        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            loop {
                let n = socket.read(&mut buffer).await.unwrap();
                if n == 0 {
                    return;
                }
                socket.write_all(&buffer[0..n]).await.unwrap();
            }
        });
    }
}

async fn client(args: Args) {
    let addr = format!("{}:{}", args.server_address.unwrap(), args.port);
    let mut socket = tokio::net::TcpStream::connect(&addr).await.unwrap();
    println!("Connected to {}", addr);

    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    loop {
        let mut buffer = [0; 1024];
        let n = stdin.read(&mut buffer).await.unwrap();
        if n == 0 {
            return;
        }
        socket.write_all(&buffer[0..n]).await.unwrap();
        let n = socket.read(&mut buffer).await.unwrap();
        stdout.write_all(&buffer[0..n]).await.unwrap();
    }
}
