use std::io::stdin;

use init_macros::create_value_init_functions;
use serde::{Deserialize, Serialize};

use crate::{
    db::structs::User,
    errors::{DBError, ServerError},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageData {
    Image(Vec<u8>),
    File(String, Vec<u8>),
    Text(String),
}
pub fn image(vec: Vec<u8>) -> MessageData {
    MessageData::Image(vec)
}
pub fn file(filename: String, vec: Vec<u8>) -> MessageData {
    MessageData::File(filename, vec)
}
pub fn text(text: String) -> MessageData {
    MessageData::Text(text)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageResponse {
    pub id: i32,
    pub user: User,
    pub content: MessageData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ErrorResponse {
    DBError(DBError),
    ServerError(ServerError),
}
// Create shorter inits like pub fn db_error(db_error) => ErrorResponse::DBError(db_error)
create_value_init_functions!(ErrorResponse, DBError(DBError), ServerError(ServerError));
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerResponse {
    Message(MessageResponse),
    AuthToken(String),
    Error(ErrorResponse),
}
// Create shorter inits like pub fn message(message_response) => ServerResponse::Message(message_response)
create_value_init_functions!(
    ServerResponse,
    Message(MessageResponse),
    AuthToken(String),
    Error(ErrorResponse)
);

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
create_value_init_functions!(
    StreamArrival,
    StreamMessage(StreamMessage),
    AuthRequest(AuthRequest),
    ReadRequest(ReadRequest),
);
