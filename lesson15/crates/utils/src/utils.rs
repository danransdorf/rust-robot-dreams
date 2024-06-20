use std::{
    fs::File,
    io::{stdin, stdout, Write},
    sync::Arc,
};

use crate::{
    db::{
        structs::{Message, User},
        DB,
    },
    errors::deserialize_object_error,
};
use anyhow::Result;
use chrono::Local;
use clap::{arg, command, Parser};
use init_macros::create_value_init_functions;
use serde::{Deserialize, Serialize};

use crate::errors::{handle_stream_error, DBError, ServerError, StreamError};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageData {
    Image(Vec<u8>),
    File(String, Vec<u8>),
    Text(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageResponse {
    id: i32,
    user: User,
    content: MessageData,
}
impl MessageResponse {
    pub fn from_db_message(message: &Message, db: &Arc<DB>) -> Result<Self, ErrorResponse> {
        let user = db.get_user(message.user_id).map_err(|e| db_error(e))?;
        let content = deserialize_data(message.content.to_owned())
            .map_err(|_| server_error(deserialize_object_error()))?;
        Ok(MessageResponse {
            id: message.id.unwrap(),
            user,
            content,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ErrorResponse {
    DBError(DBError),
    ServerError(ServerError),
}
create_value_init_functions!(ErrorResponse, DBError(DBError), ServerError(ServerError));
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerResponse {
    Message(MessageResponse),
    AuthToken(String),
    Error(ErrorResponse),
}
create_value_init_functions!(
    ServerResponse,
    Message(MessageResponse),
    AuthToken(String),
    Error(ErrorResponse)
);
impl ServerResponse {
    pub fn serialize(self) -> Vec<u8> {
        serialize_server_response(self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StreamMessage {
    pub jwt: String,
    pub message: MessageData,
}
impl StreamMessage {
    pub fn new(jwt: String, message: MessageData) -> Self {
        StreamMessage { jwt, message }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuthRequestKind {
    Login,
    Register,
}

impl AuthRequestKind {
    pub fn from_stdin() -> Self {
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "r" | "R" => AuthRequestKind::Register,
            _ => AuthRequestKind::Login,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthRequest {
    pub kind: AuthRequestKind,
    pub username: String,
    pub password: String,
}

impl AuthRequest {
    pub fn new(kind: AuthRequestKind, username: String, password: String) -> Self {
        AuthRequest {
            kind,
            username,
            password,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadRequest {
    pub jwt: String,
    pub amount: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StreamArrival {
    StreamMessage(StreamMessage),
    AuthRequest(AuthRequest),
    ReadRequest(ReadRequest),
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "localhost")]
    hostname: String,

    #[arg(long, default_value = "11111")]
    port: String,
}

pub fn get_address() -> String {
    let args = Args::parse();

    format!("{}:{}", args.hostname, args.port)
}

pub fn flush(message: &str) {
    writeln!(&mut stdout(), "{}", message).expect("Failed to write to output");
    stdout().flush().expect("Failed to flush output");
}

pub fn serialize_data(data: MessageData) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(&data)
}
pub fn deserialize_data(data: Vec<u8>) -> Result<MessageData, bincode::Error> {
    bincode::deserialize(&data)
}

pub fn serialize_server_response(data: ServerResponse) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(&data)
}
pub fn deserialize_server_response(data: Vec<u8>) -> Result<ServerResponse, bincode::Error> {
    bincode::deserialize(&data)
}

pub fn serialize_stream(stream: StreamArrival) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(&stream)
}
pub fn deserialize_stream(stream: Vec<u8>) -> Result<StreamArrival, bincode::Error> {
    bincode::deserialize(&stream)
}

static SECONDS_INDEX: usize = 19;
pub fn save_image(bytes: &Vec<u8>) -> Result<String, StreamError> {
    let timestamp = &Local::now().to_string()[..SECONDS_INDEX];
    let filename = format!("{}.png", timestamp);

    std::fs::create_dir_all("images").map_err(StreamError::FileCreationError)?;

    let mut file =
        File::create(&format!("images/{filename}")).map_err(StreamError::FileCreationError)?;
    file.write_all(&bytes)
        .map_err(StreamError::FileWriteError)?;

    Ok(filename)
}

pub fn save_file(filename: &str, bytes: &Vec<u8>) -> Result<String, StreamError> {
    std::fs::create_dir_all("files").map_err(StreamError::FileCreationError)?;

    let mut file =
        File::create(&format!("files/{filename}")).map_err(StreamError::FileCreationError)?;
    file.write_all(&bytes)
        .map_err(StreamError::FileWriteError)?;

    Ok(filename.to_string())
}

pub fn output_message_data(message_data: MessageResponse) {
    match message_data.content {
        MessageData::File(filename, bytes) => {
            match save_file(&filename, &bytes) {
                Ok(filename) => flush(&format!(
                    "{}: sent a file {}",
                    message_data.user.username, filename
                )),
                Err(e) => {
                    flush("Received file, but failed to save it");
                    handle_stream_error(e)
                }
            };
        }
        MessageData::Image(bytes) => {
            match save_image(&bytes) {
                Ok(filename) => flush(&format!(
                    "{}: sent an image {}",
                    message_data.user.username, filename
                )),
                Err(e) => {
                    flush("Received image, but failed to save it");
                    handle_stream_error(e)
                }
            };
        }
        MessageData::Text(string) => {
            flush(&format!("{}: {}", message_data.user.username, string));
        }
    }
}

pub fn unspecified_error() -> Result<(), std::io::Error> {
    return Err(std::io::Error::new(std::io::ErrorKind::Other, ""));
}
