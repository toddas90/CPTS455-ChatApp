use clap::{arg, command, Parser};
use crossterm::{cursor::MoveUp, terminal::Clear, ExecutableCommand};
use std::{error::Error, io};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::tcp::OwnedWriteHalf,
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

    // Create a new TCP listener.
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    println!("Listening on port {}", args.port);

    // Create a new message channel.
    let (tx, _) = broadcast::channel::<String>(16);

    // The main connection loop.
    loop {
        // Waits for a new connection to be established.
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

        // Copy the transmitter.
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        // Create dummy user in-case the client doesn't send a proper message.
        let username = "Anonymous".to_string() + &rand::random::<u16>().to_string();
        let dummy_user = user::User::new(&username);

        // Spawn a new task for each connection.
        tokio::spawn(async move {
            // Get the value of the socket and split it into a reader and writer.
            let mut socket = socket.unwrap();
            let (reader, mut writer) = socket.split();
            let mut reader = tokio::io::BufReader::new(reader);
            let mut line = String::new();
            let mut temp_file: message::FileMessage =
                message::FileMessage::new(&dummy_user, "temp", 0, &vec![]);

            // Main loop.
            loop {
                tokio::select! {
                    // Read a message from the client.
                    _ = reader.read_line(&mut line) => {
                        println!("Received from {}", reader.get_ref().peer_addr().unwrap());
                        if line.is_empty() {
                            break;
                        }

                        if serde_json::from_str::<message::Command>(&line).is_ok() {
                            let command: message::Command = serde_json::from_str(&line).unwrap();
                            if command.command == "/recvinfo" {
                                println!("Received command");
                                let msg = format!("{}: {} bytes -> {}\n", temp_file.username, temp_file.file_size, temp_file.file_name);
                                writer.write_all(msg.as_bytes()).await.unwrap();
                                line.clear();
                            }
                            if command.command == "/recvfile" {
                                println!("Received command");
                                let mut serialized = serde_json::to_string(&temp_file).unwrap();
                                serialized += "\n";
                                writer.write_all(serialized.as_bytes()).await.unwrap();
                                writer.write_all("Ok\n".to_string().as_bytes()).await.unwrap();
                                line.clear();
                            }
                        } else if serde_json::from_str::<message::FileMessage>(&line).is_ok() {
                            println!("Received file message");
                            temp_file = serde_json::from_str(&line).unwrap();
                            writer.write_all("Ok\n".to_string().as_bytes()).await.unwrap();

                            let message = line.clone();
                            tx.send(message).expect("Failed to send message");
                            line.clear();
                        } else if serde_json::from_str::<message::TextMessage>(&line).is_ok() {
                            println!("Received text message");
                            let message = line.clone();
                            tx.send(message).expect("Failed to send message");
                            line.clear();
                        }  else {
                            println!("Message from Telnet client; Wrappping in message struct");
                            let message = message::TextMessage::new(&dummy_user, &line, chrono::Utc::now());
                            let message = serde_json::to_string(&message).unwrap();
                            tx.send(message).expect("Failed to send message");
                            line.clear();
                        }

                        // // If the message is a valid JSON, we'll send it to the channel.
                        // // If not, we wrap it in a message before sending it.
                        // // (This is to handle telnet clients)
                        // if serde_json::from_str::<message::TextMessage>(&line).is_ok() {
                        //     let message = line.clone();
                        //     tx.send(message).expect("Failed to send message");
                        //     line.clear();
                        // } else {
                        //     println!("Message from Telnet client; Wrappping in message struct");
                        //     let message = message::TextMessage::new(&dummy_user, &line, chrono::Utc::now());
                        //     let message = serde_json::to_string(&message).unwrap();
                        //     tx.send(message).expect("Failed to send message");
                        //     line.clear();
                        // }
                    }
                    // Send the message to the other clients.
                    result = rx.recv() => {
                        let message = result.unwrap();
                        // If the message is a valid JSON, we'll send it to the client.
                        if serde_json::from_str::<message::TextMessage>(&message).is_ok() {
                            let message = serde_json::from_str::<message::TextMessage>(&message).unwrap();
                            let message = format!("{} {}: {}", message.created_at.format("%d/%m/%Y %H:%M"), message.username, message.body);
                            writer.write_all(message.as_bytes()).await.unwrap();
                        } else if serde_json::from_str::<message::FileMessage>(&message).is_ok() {
                            temp_file = serde_json::from_str::<message::FileMessage>(&message).unwrap();
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

    // Create a new TCP socket.
    let socket = tokio::net::TcpStream::connect(&addr)
        .await
        .expect("Failed to connect to server");

    println!("Connected to {}", addr);

    // Create a new user ID.
    let user_info = user::User::new(&args.username.unwrap());

    // Create a new reader and writer for the socket.
    let (reader, mut writer) = socket.into_split();
    let mut line = String::new();
    let mut reader = BufReader::new(reader).lines();

    // Recv
    tokio::spawn(async move {
        loop {
            // Read a line from the server.
            let line = reader.next_line().await.unwrap().unwrap();

            if serde_json::from_str::<message::FileMessage>(&line).is_ok() {
                let file_message = serde_json::from_str::<message::FileMessage>(&line).unwrap();
                let mut file = tokio::fs::File::create(file_message.file_name)
                    .await
                    .unwrap();
                file.write_all(&file_message.file_data).await.unwrap();
            } else {
                println!("{}", line);
            }
        }
    });

    // Send
    loop {
        // Read a line from the user and send it to the server.
        io::stdin().read_line(&mut line).unwrap();

        if line.starts_with('/') {
            match process_cmd(&line, &mut writer, &user_info).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
            line.clear();
        } else {
            // Console commands to clear the user's input.
            io::stdout().execute(crossterm::cursor::MoveUp(1)).unwrap();
            io::stdout()
                .execute(crossterm::cursor::MoveToColumn(0))
                .unwrap();
            io::stdout()
                .execute(crossterm::terminal::Clear(
                    crossterm::terminal::ClearType::CurrentLine,
                ))
                .unwrap();
            // ------------------------------
            let message = message::TextMessage::new(&user_info, &line, chrono::Utc::now());
            let mut message = serde_json::to_string(&message).unwrap();
            message += "\n";
            writer.write_all(message.as_bytes()).await.unwrap();
            line.clear();
        }
    }
}

async fn process_cmd(
    cmd: &str,
    writer: &mut OwnedWriteHalf,
    user_info: &user::User,
) -> Result<(), Box<dyn Error>> {
    let cmd = cmd.trim();
    let cmd = cmd.split_whitespace().collect::<Vec<&str>>();
    match cmd[0] {
        "/help" => {
            println!("Available commands:");
            println!("    /help - Show this message");
            println!("    /send - Send a file");
            println!("    /recvinfo - Information about the file");
            println!("    /recv - Receive a file");
            println!("    /quit - Quit the application");
            Ok(())
        }
        "/send" => {
            let file_name = cmd[1].split('/').last().unwrap();

            let file = match std::fs::read(cmd[1]) {
                Ok(file) => file,
                Err(e) => {
                    return Err(Box::new(e));
                }
            };
            let message = message::FileMessage::new(user_info, file_name, file.len(), &file);
            let mut message = serde_json::to_string(&message).unwrap();
            message += "\n";
            writer.write_all(message.as_bytes()).await.unwrap();
            Ok(())
        }
        "/recvinfo" => {
            let cmd = message::Command::new(user_info, cmd[0]);
            let mut message = serde_json::to_string(&cmd).unwrap();
            message += "\n";
            writer.write_all(message.as_bytes()).await.unwrap();
            Ok(())
        }
        "/recvfile" => {
            let cmd = message::Command::new(user_info, cmd[0]);
            let mut message = serde_json::to_string(&cmd).unwrap();
            message += "\n";
            writer.write_all(message.as_bytes()).await.unwrap();
            Ok(())
        }
        "/quit" => {
            std::process::exit(0);
        }
        _ => Err("Unknown command".into()),
    }
}
