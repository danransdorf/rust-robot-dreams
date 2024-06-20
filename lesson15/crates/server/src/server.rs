use rand::Rng;
use std::io::Error;
use std::{collections::HashMap, io::ErrorKind, net::SocketAddr, sync::Arc};
use tokio;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use utils::{
    serialize_server_response, unspecified_error, AuthRequest, AuthRequestKind, ErrorResponse,
    ServerResponse, StreamMessage,
};

use utils::errors::{handle_stream_error, ServerError, StreamError};
use utils::{deserialize_stream, serialize_data, StreamArrival};

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

use crate::db::DB;

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

static ONE_MINUTE: usize = 60;
static ONE_HOUR: usize = 60 * ONE_MINUTE;
static ONE_DAY: usize = 24 * ONE_HOUR;

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
                                handle_login(&writer, &db_clone, auth_request, &jwt_secret)
                            }
                            AuthRequestKind::Register => {
                                handle_register(&writer, &db_clone, auth_request, &jwt_secret)
                            }
                        },
                        StreamArrival::ReadRequest(read_request) => {
                            let messages = db_clone.read_history(read_request.amount).unwrap();
                            for message in messages {
                                spawn_write_task(&writer, ServerResponse::Message(message));
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

    /*  let message_data_clone = message_data.clone();
    tokio::spawn(async move {
        output_message_data(&message_data_clone);
    }); */

    Ok(stream_arrival)
}

fn handle_login(
    writer: &Arc<Mutex<WriteHalf<TcpStream>>>,
    db: &Arc<DB>,
    auth_request: AuthRequest,
    jwt_secret: &[u8; 32],
) {
    let check = db.check_password(&auth_request.username, &auth_request.password);

    let correct = match check {
        Ok(correct) => correct,
        Err(e) => {
            let response = ServerResponse::Error(ErrorResponse::DBError(e));
            spawn_write_task(&writer, response);
            return;
        }
    };

    if correct {
        let user_id = db.get_user_id(&auth_request.username).unwrap();

        let token = Claims::new(user_id, ONE_DAY).get_token(jwt_secret).unwrap();
        spawn_write_task(&writer, ServerResponse::AuthToken(token));
    } else {
        spawn_write_task(
            &writer,
            ServerResponse::Error(ErrorResponse::ServerError(ServerError::InvalidCredentials)),
        );
    }
}

fn handle_register(
    writer: &Arc<Mutex<WriteHalf<TcpStream>>>,
    db: &Arc<DB>,
    auth_request: AuthRequest,
    jwt_secret: &[u8; 32],
) {
    match db.create_user(auth_request.username, auth_request.password) {
        Ok(new_user) => {
            let token = Claims::new(new_user.id, ONE_DAY)
                .get_token(jwt_secret)
                .unwrap();

            spawn_write_task(&writer, ServerResponse::AuthToken(token));
        }
        _ => spawn_write_task(
            &writer,
            ServerResponse::Error(ErrorResponse::ServerError(ServerError::UsernameUsed)),
        ),
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
        Ok(claims) => claims.user_id,
        _ => {
            eprintln!("Invalid token");
            spawn_write_task(
                &writer,
                ServerResponse::Error(ErrorResponse::ServerError(ServerError::InvalidToken)),
            );
            return;
        }
    };

    db.save_message(user_id, stream_message.message.clone())
        .unwrap();

    for (addr, client) in clients.lock().await.iter() {
        if *addr != client_addr {
            match Claims::from_token(&client.token, jwt_secret) {
                Ok(_) => {
                    spawn_write_task(&client.writer, todo!());
                }
                _ => (), // I could message the client that the token has expired, but messaging on every message is contraproductive
            }
        }
    }

    return;
}
