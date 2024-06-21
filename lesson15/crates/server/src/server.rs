use rand::Rng;
use std::io::Error;
use std::{collections::HashMap, io::ErrorKind, net::SocketAddr, sync::Arc};
use tokio;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use utils::{
    auth_token, db_error, error, message, server_error, AuthRequest, AuthRequestKind,
    MessageResponse, ServerResponse, StreamMessage,
};

use utils::errors::{
    handle_stream_error, invalid_credentials, invalid_token, username_used, StreamError,
};
use utils::{deserialize_stream, StreamArrival};

type WriteHalfArc = Arc<Mutex<WriteHalf<TcpStream>>>;
struct Client {
    writer: WriteHalfArc,
    token: String,
}
impl Client {
    pub fn new(writer: WriteHalfArc, token: String) -> Self {
        Client { writer, token }
    }
}
type StreamsHashMap = HashMap<SocketAddr, Client>;

use utils::db::DB;

use jsonwebtoken::{
    decode, encode, get_current_timestamp, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

static ONE_MINUTE: u64 = 60;
static ONE_HOUR: u64 = 60 * ONE_MINUTE;
static ONE_DAY: u64 = 24 * ONE_HOUR;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /** User ID */
    pub sub: i32,
    pub exp: u64,
}

impl Claims {
    pub fn new(user_id: i32, exp: u64) -> Self {
        Claims { sub: user_id, exp }
    }
    pub fn from_token(token: &str, secret: &[u8]) -> Result<Self, jsonwebtoken::errors::Error> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)?;
        Ok(token_data.claims)
    }
    pub fn get_token(&self, secret: &[u8]) -> Result<String, jsonwebtoken::errors::Error> {
        let header = Header::new(Algorithm::HS256);
        let token = encode(&header, &self, &EncodingKey::from_secret(secret))?;
        Ok(token)
    }
}

pub async fn start_server(address: String) {
    let mut jwt_secret = [0u8; 32];
    rand::thread_rng().fill(&mut jwt_secret);

    let db = Arc::new(DB::new().unwrap());

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
            .insert(client_addr, Client::new(Arc::clone(&writer), String::new()));

        println!("Stream opened (addr: {})", client_addr);

        let db_clone = Arc::clone(&db);
        let clients_clone = Arc::clone(&clients);
        tokio::spawn(async move {
            loop {
                match handle_stream(&reader).await {
                    Ok(stream_arrival) => match stream_arrival {
                        StreamArrival::StreamMessage(stream_message) => {
                            handle_stream_message(
                                stream_message,
                                &writer,
                                &clients_clone,
                                client_addr,
                                &db_clone,
                                &jwt_secret,
                            )
                            .await;
                        }
                        StreamArrival::AuthRequest(auth_request) => match auth_request.kind {
                            AuthRequestKind::Login => {
                                if let Some(token) =
                                    handle_login(&writer, &db_clone, auth_request, &jwt_secret)
                                {
                                    clients_clone
                                        .lock()
                                        .await
                                        .get_mut(&client_addr)
                                        .unwrap()
                                        .token = token;
                                }
                            }
                            AuthRequestKind::Register => {
                                if let Some(token) =
                                    handle_register(&writer, &db_clone, auth_request, &jwt_secret)
                                {
                                    clients_clone
                                        .lock()
                                        .await
                                        .get_mut(&client_addr)
                                        .unwrap()
                                        .token = token;
                                }
                            }
                        },
                        StreamArrival::ReadRequest(read_request) => {
                            let messages = db_clone.read_history(read_request.amount).unwrap();
                            for message_obj in messages {
                                let message_response_res =
                                    MessageResponse::from_db_message(&message_obj, &db_clone);

                                match message_response_res {
                                    Ok(message_response) => {
                                        spawn_write_task(&writer, message(message_response));
                                    }
                                    Err(error_response) => {
                                        spawn_write_task(&writer, error(error_response));
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

fn spawn_write_task(writer: &Arc<Mutex<WriteHalf<TcpStream>>>, response: ServerResponse) {
    let content = response.serialize();
    let writer = Arc::clone(writer);
    tokio::spawn(async move {
        write_into_stream(&writer, &content)
            .await
            .map_err(|e| println!("{}", e))
            .ok();
    });
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

    Ok(stream_arrival)
}

fn handle_login(
    writer: &Arc<Mutex<WriteHalf<TcpStream>>>,
    db: &Arc<DB>,
    auth_request: AuthRequest,
    jwt_secret: &[u8; 32],
) -> Option<String> {
    let check = db.check_password(&auth_request.username, &auth_request.password);

    let correct = match check {
        Ok(correct) => correct,
        Err(e) => {
            let response = error(db_error(e));
            spawn_write_task(&writer, response);
            return None;
        }
    };

    if correct {
        let user_id = db.get_user_id(&auth_request.username).unwrap();

        let token = Claims::new(user_id, get_current_timestamp() + ONE_DAY)
            .get_token(jwt_secret)
            .unwrap();

        spawn_write_task(&writer, auth_token(token.clone()));

        return Some(token);
    }

    spawn_write_task(&writer, error(server_error(invalid_credentials())));

    return None;
}

fn handle_register(
    writer: &Arc<Mutex<WriteHalf<TcpStream>>>,
    db: &Arc<DB>,
    auth_request: AuthRequest,
    jwt_secret: &[u8; 32],
) -> Option<String> {
    match db.create_user(auth_request.username, auth_request.password) {
        Ok(new_user) => {
            let token = Claims::new(new_user.id.unwrap(), get_current_timestamp() + ONE_DAY)
                .get_token(jwt_secret)
                .unwrap();

            spawn_write_task(&writer, auth_token(token.clone()));

            Some(token)
        }
        Err(e) => {
            println!("{}", e);
            spawn_write_task(&writer, error(server_error(username_used())));
            None
        }
    }
}

async fn handle_stream_message(
    stream_message: StreamMessage,
    writer: &Arc<Mutex<WriteHalf<TcpStream>>>,
    clients: &Arc<Mutex<StreamsHashMap>>,
    client_addr: SocketAddr,
    db: &Arc<DB>,
    jwt_secret: &[u8; 32],
) {
    let user_id = match Claims::from_token(&stream_message.jwt, jwt_secret) {
        Ok(claims) => claims.sub,
        _ => {
            eprintln!("Invalid token");
            spawn_write_task(&writer, error(server_error(invalid_token())));
            return;
        }
    };

    let message_obj = db
        .save_message(user_id, stream_message.message.clone())
        .unwrap();

    for (addr, client) in clients.lock().await.iter() {
        if *addr != client_addr {
            println!("Sending message to {}, token: {}", addr, &client.token);
            match Claims::from_token(&client.token, jwt_secret) {
                Ok(_) => match MessageResponse::from_db_message(&message_obj, &db) {
                    Ok(message_response) => {
                        println!("Should send message");
                        spawn_write_task(&client.writer, message(message_response));
                    }
                    Err(error_response) => spawn_write_task(&writer, error(error_response)),
                },
                _ => (), // I could message the client that the token has expired, but messaging on every message that passes through the chat seems counterproductive
            }
        }
    }

    return;
}
