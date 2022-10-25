use clap::{arg, command, Parser};
use std::{error::Error, io};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::broadcast,
};

pub mod message;
pub mod user;

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

    let (tx, _) = broadcast::channel::<String>(16);

    loop {
        let socket = match listener.accept().await {
            Ok((socket, _addr)) => {
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

        // Create dummy user in-case the client doesn't send a proper message.
        let username = "Anonymous".to_string() + &rand::random::<u16>().to_string();
        let dummy_user = user::User::new(&username);

        tokio::spawn(async move {
            let mut socket = socket.unwrap();
            let (reader, mut writer) = socket.split();

            let mut reader = tokio::io::BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    _ = reader.read_line(&mut line) => {
                        println!("Received msg from {}", reader.get_ref().peer_addr().unwrap());
                        if line.is_empty() {
                            break;
                        }

                        if serde_json::from_str::<message::Message>(&line).is_ok() {
                            let message = line.clone();
                            tx.send(message).expect("Failed to send message");
                            line.clear();
                        } else {
                            let message = message::Message::new(&dummy_user, &line, chrono::Utc::now());
                            let message = serde_json::to_string(&message).unwrap();
                            tx.send(message).expect("Failed to send message");
                            line.clear();
                        }
                    }
                    result = rx.recv() => {
                        let message = result.unwrap();
                        if serde_json::from_str::<message::Message>(&message).is_ok() {
                            let message = serde_json::from_str::<message::Message>(&message).unwrap();
                            let message = format!("{} {}: {}", message.created_at.timestamp(), message.username, message.body);
                            writer.write_all(message.as_bytes()).await.unwrap();
                        } else {
                            writer.write_all(message.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}

async fn client(args: Args) -> Result<(), Box<dyn Error>> {
    let addr = format!("{}:{}", args.server_address.unwrap(), args.port);
    let socket = tokio::net::TcpStream::connect(&addr)
        .await
        .expect("Failed to connect to server");
    println!("Connected to {}", addr);
    let user_info = user::User::new(&args.username.unwrap());

    let (reader, mut writer) = socket.into_split();

    let mut line = String::new();

    let mut reader = BufReader::new(reader).lines();

    // Recv
    tokio::spawn(async move {
        loop {
            let line = reader.next_line().await.unwrap().unwrap();
            println!("{}", line);
        }
    });

    // Send
    loop {
        io::stdin().read_line(&mut line).unwrap();
        let message = message::Message::new(&user_info, &line, chrono::Utc::now());
        let mut message = serde_json::to_string(&message).unwrap();
        message += "\n";
        writer.write_all(message.as_bytes()).await.unwrap();
        line.clear();
    }
}
