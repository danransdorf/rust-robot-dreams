use std::{
    collections::BTreeMap,
    io::{stdin, Read},
    net::TcpStream,
    ops::Deref,
    sync::{Arc, Mutex},
};

use base64::{engine::general_purpose, Engine as _};
use utils::{
    auth_request, deserialize_server_response, message_request, read_request, text, AuthRequest,
    AuthRequestKind, ErrorResponse, MessageContent, MessageRequest, MessageResponse,
    ServerResponse,
};
use utils::{errors::ClientError, ReadRequest};
use utils::{
    flush,
    write_utils::{handle_file, handle_image, serialize_and_write},
};
use yew::{platform::spawn_local, prelude::*};

/// Prints the help message
fn print_help() {
    println!("Possible commands:");
    println!("    .file <path> - Send a file located at <path> to the chat");
    println!("    .image <path> - Send an image located at <path> to the chat");
    println!("    .quit - Exit the chat application");
    println!("    .history <amount> - Display the last <amount> messages from the chat history");
    println!("    .help - Display this help message");
}

/// Exits the program
fn exit_program() {
    println!("Exiting program...");
    std::process::exit(0x0100);
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub address: String,
}

#[function_component(App)]
fn app(props: &Props) -> Html {
    let address = props.address.clone();
    println!("Creating a client on address: {}", address);

    let stream =
        TcpStream::connect(&address).expect("Failed to connect to the server, is the server live?");
    let mut stream_clone = stream.try_clone().unwrap();

    let jwt: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let jwt_clone = Arc::clone(&jwt);

    let messages = Arc::new(Mutex::new(use_state(|| {
        BTreeMap::<i32, MessageResponse>::new()
    })));
    let messages_a = messages.clone();

    spawn_local(async move {
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
                    let mut jwt_lock = jwt.lock().unwrap();
                    *jwt_lock = Some(token);

                    flush("Successfully logged in");
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
                }
                ServerResponse::Message(message) => {
                    let mut messages_clone = messages_a.lock().unwrap().clone().deref().to_owned();
                    messages_clone.insert(message.id, message);
                    messages_a.lock().unwrap().set(messages_clone);
                }
            }
        }
    });

    spawn_local(async move {
        loop {
            let jwt_lock = jwt_clone.lock().unwrap().clone();
            match &jwt_lock {
                None => {
                    let auth_request_content = prompt_auth();

                    serialize_and_write(&stream, auth_request(auth_request_content))
                        .map_err(|e| eprintln!("{}", e))
                        .ok();
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
                        Some(".history") => {
                            if let Some(amount_string) = command.next() {
                                if let Ok(amount) = amount_string.parse::<i32>() {
                                    serialize_and_write(
                                        &stream,
                                        read_request(ReadRequest {
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
                                message_request(MessageRequest::new(
                                    jwt_token.to_owned(),
                                    text(input_string),
                                )),
                            )
                            .map_err(|e| eprintln!("{}", e))
                            .ok();
                        }
                    }
                }
            }
        }
    });

    

    html! {
        <div>
            <h1>{"Chat App"}</h1>
            <p>{"Welcome to the chat app!"}</p>
            <div class="flex flex-col gap-4 items-center">
                <div class="flex flex-col gap-4 items-center">
                    <h2>{"Messages"}</h2>
                    <div class="flex flex-col gap-4 items-center">
                        {
                            messages.lock().unwrap().iter().map(|(_, message)| {
                                html! {
                                    <div class="flex flex-col gap-2 items-center">
                                        <p>{match &message.content {
                                            MessageContent::Text(text) => html!{text},
                                            MessageContent::Image(bytes) => {
                                                html!{<img src={"data:image/png;base64,".to_owned() + &general_purpose::STANDARD.encode(bytes)} alt="img"/>}},
                                            MessageContent::File(filename, file_bytes) => {
                                                let mime_type = mime_guess::from_path(&filename).first_or_octet_stream();
                                                let base64_encoded = &general_purpose::STANDARD.encode(file_bytes);
                                                html!{<a href={format!("data:{};base64,{}", mime_type, base64_encoded)} download={filename.to_owned()}></a>}},
                                        }}</p>
                                        <p>{&message.user.username}</p>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </div>
                <div class="flex flex-col gap-4 items-center">
                    <h2>{"Send a message"}</h2>
                    <input type="text" placeholder="Enter message" />
                    <button>{"Send"}</button>
                </div>
            </div>
        </div>
    }
}



pub fn run_app(address: String) {
    yew::Renderer::<App>::with_props(Props { address }).render();
}

/// Prompts the user for authentication details
fn prompt_auth() -> AuthRequest {
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

    AuthRequest::new(
        auth_method,
        username.trim().to_string(),
        password.trim().to_string(),
    )
}
