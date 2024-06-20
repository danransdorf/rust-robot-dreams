use std::{
    io::{stdin, Read},
    net::TcpStream,
    sync::Arc,
};
use tokio::{
    sync::{Mutex, Notify},
    time::{sleep, Duration},
};

use utils::{
    deserialize_server_response, output_message_data, AuthRequest, AuthRequestKind, ErrorResponse,
    ServerResponse, StreamArrival, StreamMessage,
};
use utils::{errors::ClientError, ReadRequest};
use utils::{
    flush,
    write_utils::{handle_file, handle_image, serialize_and_write},
    MessageData,
};

fn print_help() {
    println!("Possible commands:");
    println!("    .file <path>");
    println!("    .image <path>");
    println!("    .quit");
    println!("    .help");
}
fn exit_program() {
    println!("Exiting program...");
    std::process::exit(0x0100);
}

pub async fn start_client(address: String) {
    println!("Creating a client on address: {}", address);

    let stream = TcpStream::connect(&address).unwrap();
    let mut stream_clone = stream.try_clone().unwrap();

    let jwt: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let jwt_clone = Arc::clone(&jwt);

    let notify = Arc::new(Notify::new());
    let notify_a = Arc::clone(&notify);
    let notify_b = Arc::clone(&notify);

    tokio::spawn(async move {
        loop {
            let mut len_buffer = [0; 4];

            stream_clone
                .read_exact(&mut len_buffer)
                .map_err(|_| {
                    println!("stream has been closed");
                    std::process::exit(0x0100);
                })
                .ok();

            let len = u32::from_be_bytes(len_buffer);
            let mut buffer = vec![0; len as usize];
            stream_clone
                .read_exact(&mut buffer)
                .map_err(|e| eprintln!("{}", e))
                .ok();

            let message_data = match deserialize_server_response(buffer) {
                Ok(data) => data,
                Err(e) => {
                    println!("Failed to deserialize server response: {e}");
                    continue;
                }
            };

            match message_data {
                ServerResponse::AuthToken(token) => {
                    let mut jwt_lock = jwt.lock().await;
                    *jwt_lock = Some(token);

                    flush("Successfully logged in");
                    notify_a.notify_one();
                    sleep(Duration::from_millis(1)).await;
                }
                ServerResponse::Error(error) => {
                    match error {
                        ErrorResponse::DBError(e) => {
                            eprintln!("DB error: {e}");
                        }
                        ErrorResponse::ServerError(e) => {
                            eprintln!("Server error: {e}");
                        }
                    };
                    println!("trying to notify");
                    notify_a.notify_one();
                    sleep(Duration::from_millis(1)).await;
                    println!("prolly notified");
                }
                ServerResponse::Message(message_data) => {
                    output_message_data(&message_data);
                }
            }
        }
    });

    tokio::spawn(async move {
        loop {
            let jwt_lock = jwt_clone.lock().await.clone();
            match &jwt_lock {
                None => {
                    let (auth_method, (username, password)) = prompt_auth();

                    serialize_and_write(
                        &stream,
                        StreamArrival::AuthRequest(AuthRequest::new(
                            auth_method,
                            username,
                            password,
                        )),
                    )
                    .map_err(|e| eprintln!("{}", e))
                    .ok();

                    println!("waiting");
                    notify_b.notified().await;
                    println!("continuing")
                }
                Some(jwt_token) => {
                    flush("\nEnter message (Ctrl+D to send), send `.help` for possible commands:");

                    let mut input_bytes = vec![];
                    if let Err(e) = stdin().read_to_end(&mut input_bytes) {
                        eprintln!("\nFailed to read the input: {}", e);
                        continue;
                    }
                    flush("\n");

                    let input_string = String::from_utf8_lossy(&input_bytes).trim().to_string();
                    let mut command = input_string.splitn(2, ' ');
                    match command.next() {
                        Some(".quit") => exit_program(),
                        Some(".help") => print_help(),
                        Some(".image") => {
                            if let Some(path) = command.next() {
                                handle_image(&stream, path, jwt_token.to_owned())
                                    .map_err(|e| println!("{}", e))
                                    .ok();
                            } else {
                                eprintln!("{}", ClientError::PathError(".image"))
                            }
                        }
                        Some(".file") => {
                            if let Some(path) = command.next() {
                                handle_file(&stream, path, jwt_token.to_owned())
                                    .map_err(|e| println!("{}", e))
                                    .ok();
                            } else {
                                eprintln!("{}", ClientError::PathError(".file"))
                            }
                        }
                        Some(".read") => {
                            if let Some(amount_string) = command.next() {
                                if let Ok(amount) = amount_string.parse::<i32>() {
                                    serialize_and_write(
                                        &stream,
                                        StreamArrival::ReadRequest(ReadRequest {
                                            jwt: jwt_token.to_owned(),
                                            amount,
                                        }),
                                    )
                                    .map_err(|e| eprintln!("{}", e))
                                    .ok();
                                } else {
                                    eprintln!("Invalid amount provided");
                                }
                            } else {
                                eprintln!("No amount provided");
                            }
                        }
                        _ => {
                            serialize_and_write(
                                &stream,
                                StreamArrival::StreamMessage(StreamMessage::new(
                                    jwt_token.to_owned(),
                                    MessageData::Text(input_string),
                                )),
                            )
                            .map_err(|e| eprintln!("{}", e))
                            .ok();
                        }
                    }
                }
            }
        }
    })
    .await
    .unwrap();
}

fn prompt_auth() -> (AuthRequestKind, (String, String)) {
    println!("Do you want to log in or register? [L/r]");
    let auth_method = AuthRequestKind::from_stdin();

    match auth_method {
        AuthRequestKind::Login => {
            println!("Log In")
        }
        AuthRequestKind::Register => {
            println!("Register")
        }
    }

    let mut username = String::new();
    let mut password = String::new();

    println!("Enter username:");
    stdin().read_line(&mut username).unwrap();
    println!("Enter password:");
    stdin().read_line(&mut password).unwrap();

    (
        auth_method,
        (username.trim().to_string(), password.trim().to_string()),
    )
}
