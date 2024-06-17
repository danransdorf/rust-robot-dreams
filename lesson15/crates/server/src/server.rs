use rand::Rng;
use std::io::Error;
use std::{collections::HashMap, io::ErrorKind, net::SocketAddr, sync::Arc};
use tokio;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use utils::AuthRequestKind;

use utils::errors::{handle_stream_error, StreamError};
use utils::{deserialize_stream, output_message_data, serialize_data, MessageData, StreamArrival};

type WriteHalfArc = Arc<Mutex<WriteHalf<TcpStream>>>;
type StreamsHashMap = HashMap<SocketAddr, WriteHalfArc>;

use crate::db::DB;

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub user_id: i32,
    pub exp: usize,
}

impl Claims {
    pub fn new(user_id: i32, exp: usize) -> Self {
        Claims { user_id, exp }
    }
    pub fn from_token(token: &str, secret: &[u8]) -> Result<Self, jsonwebtoken::errors::Error> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)?;
        Ok(token_data.claims)
    }
    pub fn get_token(&self, secret: &[u8]) -> Result<String, jsonwebtoken::errors::Error> {
        let token = encode(&Header::default(), &self, &EncodingKey::from_secret(secret))?;
        Ok(token)
    }
}

pub async fn start_server(address: String) {
    let mut jwt_secret = [0u8; 32];
    rand::thread_rng().fill(&mut jwt_secret);

    let db = Arc::new(DB::new().unwrap());
    /*
    DB::new() -> Result<DB>: Creates a new database connection.
    DB::create_user(&self, username: String, password: String) -> Result<User>: Creates a new user.
    DB::check_password(&self, username: &str, password: &str) -> Result<bool>: Checks a user's password.
    DB::save_message(&self, user_id: i32, message: MessageData) -> Result<()>: Saves a message.
    DB::read_message(&self, message_id: i32) -> Result<Message>: Reads a message by ID.
    DB::read_history(&self, amount: i32) -> Result<Vec<Message>>: Reads the most recent messages.*/

    println!("Creating a server on address: {}", address);
    let listener = TcpListener::bind(address).await.unwrap();

    let clients: Arc<Mutex<StreamsHashMap>> = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (stream, client_addr) = listener.accept().await.unwrap();
        let (rd, wr) = io::split(stream);
        let reader = Arc::new(Mutex::new(rd));
        let writer = Arc::new(Mutex::new(wr));

        clients
            .lock()
            .await
            .insert(client_addr, Arc::clone(&writer));

        println!("Stream opened (addr: {})", client_addr);

        let db_clone = Arc::clone(&db);
        let clients_clone = Arc::clone(&clients);
        tokio::spawn(async move {
            loop {
                match handle_stream(&reader).await {
                    Ok(stream_arrival) => match stream_arrival {
                        StreamArrival::StreamMessage(stream_message) => {
                            let user_id = match Claims::from_token(&stream_message.jwt, &jwt_secret)
                            {
                                Ok(claims) => claims.user_id,
                                _ => {
                                    eprintln!("Invalid token");
                                    break;
                                }
                            };

                            db_clone
                                .save_message(user_id, stream_message.message.clone())
                                .unwrap();

                            let serialized_string = match serialize_data(stream_message.message) {
                                Ok(string) => string,
                                _ => {
                                    eprintln!("Unable to serialize object");
                                    continue;
                                }
                            };

                            for (addr, client_writer) in clients_clone.lock().await.iter() {
                                if *addr != client_addr {
                                    let client_writer = client_writer.clone();
                                    let serialized_string = serialized_string.clone();

                                    tokio::spawn(async move {
                                        write_into_stream(&client_writer, &serialized_string)
                                            .await
                                            .map_err(|e| println!("{}", e))
                                            .ok();
                                    });
                                }
                            }
                        }
                        StreamArrival::AuthRequest(auth_request) => match auth_request.kind {
                            AuthRequestKind::Login => {
                                let correct = db_clone
                                    .check_password(&auth_request.username, &auth_request.password)
                                    .unwrap();

                                if correct {
                                    let user_id =
                                        db_clone.get_user_id(&auth_request.username).unwrap();

                                    for (addr, client_writer) in clients_clone.lock().await.iter() {
                                        if *addr == client_addr {
                                            let token = Claims::new(user_id, 60 * 60 * 24)
                                                .get_token(&jwt_secret)
                                                .unwrap();
                                            tokio::spawn(async move {
                                                write_into_stream(
                                                    &client_writer,
                                                    &token.as_bytes(),
                                                )
                                                .await
                                                .map_err(|e| println!("{}", e))
                                                .ok();
                                            });
                                        }
                                    }
                                } else {
                                    for (addr, client_writer) in clients_clone.lock().await.iter() {
                                        if *addr == client_addr {
                                            tokio::spawn(async move {
                                                write_into_stream(
                                                    &client_writer,
                                                    b"Invalid credentials",
                                                )
                                                .await
                                                .map_err(|e| println!("{}", e))
                                                .ok();
                                            });
                                        }
                                    }
                                }
                            }
                            AuthRequestKind::Register => {
                                let new_user = match db_clone
                                    .create_user(auth_request.username, auth_request.password)
                                {
                                    Ok(user) => user,
                                    _ => {
                                        for (addr, client_writer) in
                                            clients_clone.lock().await.iter()
                                        {
                                            if *addr == client_addr {
                                                tokio::spawn(async move {
                                                    write_into_stream(
                                                        &client_writer,
                                                        b"Username already exists",
                                                    )
                                                    .await
                                                    .map_err(|e| println!("{}", e))
                                                    .ok();
                                                });
                                            }
                                        }
                                        continue;
                                    }
                                };

                                for (addr, client_writer) in clients_clone.lock().await.iter() {
                                    if *addr == client_addr {
                                        let token = Claims::new(new_user.id, 60 * 60 * 24)
                                            .get_token(&jwt_secret)
                                            .unwrap();
                                        tokio::spawn(async move {
                                            write_into_stream(&client_writer, &token.as_bytes())
                                                .await
                                                .map_err(|e| println!("{}", e))
                                                .ok();
                                        });
                                    }
                                }
                            }
                        },
                        StreamArrival::ReadRequest(read_request) => {
                            let messages = db_clone.read_history(read_request.amount).unwrap();

                            for message in messages {
                                for (addr, client_writer) in clients_clone.lock().await.iter() {
                                    if *addr == client_addr {
                                        let content = message.content.clone();
                                        tokio::spawn(async move {
                                            write_into_stream(&client_writer, &content)
                                                .await
                                                .map_err(|e| println!("{}", e))
                                                .ok();
                                        });
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => match e {
                        StreamError::StreamClosed => {
                            eprintln!("Stream has been closed (addr: {})", &client_addr);
                            clients_clone.lock().await.remove(&client_addr);
                            break;
                        }
                        _ => handle_stream_error(e),
                    },
                }
            }
        });
    }
}

async fn write_into_stream(
    writer: &Arc<Mutex<WriteHalf<TcpStream>>>,
    content: &[u8],
) -> std::io::Result<()> {
    let len_bytes = (content.len() as u32).to_be_bytes();

    let mut locked_writer = writer.lock().await;
    locked_writer.write(&len_bytes).await?;
    locked_writer.write_all(content).await?;

    Ok(())
}

async fn handle_stream(
    reader: &Arc<Mutex<ReadHalf<TcpStream>>>,
) -> Result<StreamArrival, StreamError> {
    let mut locked_reader = reader.lock().await;

    let mut len_buffer = [0; 4];
    locked_reader
        .read_exact(&mut len_buffer)
        .await
        .map_err(|_| StreamError::StreamClosed)?;

    let len = u32::from_be_bytes(len_buffer);

    let mut buffer = vec![0; len as usize];
    locked_reader
        .read_exact(&mut buffer)
        .await
        .map_err(StreamError::ReadMessageError)?;

    drop(locked_reader); // Manually drop just to be sure

    let stream_arrival = deserialize_stream(buffer).map_err(|_| {
        StreamError::ReadMessageError(Error::new(ErrorKind::InvalidData, "Failed to deserialize"))
    })?;

    /*  let message_data_clone = message_data.clone();
    tokio::spawn(async move {
        output_message_data(&message_data_clone);
    }); */

    Ok(stream_arrival)
}
