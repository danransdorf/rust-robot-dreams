use std::io::stdin;

use init_macros::create_valueenum_init_functions;
use serde::{Deserialize, Serialize};

use crate::{
    db::structs::User,
    errors::{DBError, ServerError},
};

/// Represents the content of a chat message.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageContent {
    Image(Vec<u8>),
    File(String, Vec<u8>),
    Text(String),
}

// My macros don't support generics, so I have to manually implement these init functions
/// Equivalent to writing out MessageContent::Image(vec)
pub fn image(vec: Vec<u8>) -> MessageContent {
    MessageContent::Image(vec)
}
/// Equivalent to writing out MessageContent::File(filename, vec)
pub fn file(filename: String, vec: Vec<u8>) -> MessageContent {
    MessageContent::File(filename, vec)
}
/// Equivalent to writing out MessageContent::Text(text)
pub fn text(text: String) -> MessageContent {
    MessageContent::Text(text)
}

/// Response variant for a message from the server
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageResponse {
    pub id: i32,
    pub user: User,
    pub content: MessageContent,
}

/// Response variant for an error from the server
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ErrorResponse {
    DBError(DBError),
    ServerError(ServerError),
}
// Create shorter inits like pub fn db_error(db_error) => ErrorResponse::DBError(db_error)
create_valueenum_init_functions!(ErrorResponse, DBError(DBError), ServerError(ServerError));

/// Represents a response coming from the server
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerResponse {
    Message(MessageResponse),
    AuthToken(String),
    Error(ErrorResponse),
}
// Create shorter inits like pub fn message(message_response) => ServerResponse::Message(message_response)
create_valueenum_init_functions!(
    ServerResponse,
    Message(MessageResponse),
    AuthToken(String),
    Error(ErrorResponse)
);

/// Request variant for sending messages
///
/// # Fields
/// * `jwt` - The JWT token of the user
/// * `message` - The content of the message
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageRequest {
    pub jwt: String,
    pub message: MessageContent,
}
impl MessageRequest {
    pub fn new(jwt: String, message: MessageContent) -> Self {
        MessageRequest { jwt, message }
    }
}

/// Type of authentication request
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

/// Request variant for authentication
///
/// # Fields
/// * `kind` - The type of authentication request
/// * `username` - The username of the user
/// * `password` - The password of the user
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

/// Request variant for reading message history
///
/// # Fields
/// * `jwt` - The JWT token of the user
/// * `amount` - The number of messages to read
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadRequest {
    pub jwt: String,
    pub amount: i32,
}

/// Represents a request to the server
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StreamRequest {
    MessageRequest(MessageRequest),
    AuthRequest(AuthRequest),
    ReadRequest(ReadRequest),
}
create_valueenum_init_functions!(
    StreamRequest,
    MessageRequest(MessageRequest),
    AuthRequest(AuthRequest),
    ReadRequest(ReadRequest),
);
