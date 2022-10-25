use std::{error::Error, net::Ipv4Addr};

use clap::{arg, command, Parser};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
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

    send_recv(socket.unwrap(), args.username.unwrap()).await
}

async fn client(args: Args) -> Result<(), Box<dyn Error>> {
    let addr = format!("{}:{}", args.server_address.unwrap(), args.port);
    let socket = tokio::net::TcpStream::connect(&addr).await.unwrap();
    println!("Connected to {}", addr);

    send_recv(socket, args.username.unwrap()).await
}

async fn send_recv(socket: TcpStream, username: String) -> Result<(), Box<dyn Error>> {
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let (mut reader, mut writer) = socket.into_split();

    // Sending Code
    tokio::spawn(async move {
        let mut buffer = [0; 2048];
        loop {
            print!("> ");
            let bytes_read = stdin.read(&mut buffer).await.unwrap();
            if bytes_read == 0 {
                return;
            }

            // If the message starts with "file::", treat it as a file path. Otherwise, treat it as a message.
            if buffer.starts_with(b"file::") {
                let path = String::from_utf8_lossy(&buffer[6..bytes_read - 1]);
                let file = tokio::fs::read(path.as_ref()).await.unwrap();
                writer.write_all(&file).await.unwrap();
            } else {
                let message = message::Message::new(
                    username.as_ref(),
                    String::from_utf8_lossy(&buffer[..bytes_read]).as_ref(),
                    chrono::Utc::now(),
                );

                let encoded = bincode::serialize(&message).unwrap();

                writer.write_all(&encoded).await.unwrap();
            }
        }
    });

    // Receiving Code
    let mut buffer = [0; 2048];
    loop {
        let bytes_read = reader.read(&mut buffer).await.unwrap();
        if bytes_read == 0 {
            continue;
        }

        let message: message::Message = bincode::deserialize(&buffer[..bytes_read]).unwrap();

        stdout
            .write_all(format!("{}", message).as_bytes())
            .await
            .unwrap();
    }
}
