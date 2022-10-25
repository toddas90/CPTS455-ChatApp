use clap::{arg, command, Parser};
use std::error::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    sync::broadcast,
};

pub mod message;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // If the port is not specified, we'll use 6969.
    #[arg(short, long, default_value_t = 6969)]
    port: u16,

    // If the host is not specified, we'll assume this user wants to create the server.
    #[arg(short, long)]
    server_address: Option<String>,

    // If the username is not specified, we'll use "Anonymous".
    #[arg(short, long, default_value = "Anonymous")]
    username: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.server_address.is_none() {
        server(args).await?;
    } else {
        client(args).await?;
    }
    Ok(())
}

async fn server(args: Args) -> Result<(), Box<dyn Error>> {
    let addr = format!("0.0.0.0:{}", args.port);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to address {}", addr));

    println!("Listening on {}", listener.local_addr().unwrap());

    let username = args.username.unwrap();

    let (tx, _) = broadcast::channel::<String>(10);

    loop {
        let socket = match listener.accept().await {
            Ok((socket, _)) => {
                println!("New connection: {}", socket.peer_addr().unwrap());
                Ok(socket)
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                Err(e)
            }
        };

        let tx = tx.clone();
        let mut rx = tx.subscribe();
        let username = username.clone();

        tokio::spawn(async move {
            let mut socket = socket.unwrap();
            let (reader, mut writer) = socket.split();

            let mut reader = tokio::io::BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    _ = reader.read_line(&mut line) => {
                        if line.is_empty() {
                            break;
                        }

                        let message = message::Message::new(&username, &line, chrono::Utc::now());
                        let message = serde_json::to_string(&message).unwrap();
                        tx.send(message.to_string()).expect("Failed to send message");
                        line.clear();
                    }
                    result = rx.recv() => {
                        let message = result.unwrap();
                        let message = serde_json::from_str::<message::Message>(&message).unwrap();
                        let message = format!("{} {}: {}", message.created_at.timestamp(), message.user, message.body);
                        writer.write_all(message.as_bytes()).await.unwrap();
                    }
                }
            }
        });
    }
}

async fn client(args: Args) -> Result<(), Box<dyn Error>> {
    let addr = format!("{}:{}", args.server_address.unwrap(), args.port);
    let mut socket = tokio::net::TcpStream::connect(&addr).await.unwrap();
    println!("Connected to {}", addr);

    let (reader, mut writer) = socket.split();

    let mut reader = tokio::io::BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            Err("Connection closed")?;
        }

        println!("{}", line);
        writer.write_all(line.as_bytes()).await?;
    }
}
