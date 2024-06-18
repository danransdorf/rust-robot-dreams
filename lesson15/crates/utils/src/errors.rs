use serde::{Deserialize, Serialize};
use std::io::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StreamError {
    #[error("Failed to create file")]
    FileCreationError(Error),

    #[error("Failed to write to file")]
    FileWriteError(Error),

    #[error("Stream has been closed")]
    StreamClosed,

    #[error("Failed to read message")]
    ReadMessageError(Error),
}

pub fn invalid_input_error(error: &'static str) -> Error {
    Error::new(std::io::ErrorKind::InvalidInput, error)
}

pub fn handle_stream_error(e: StreamError) {
    eprintln!("{}", e);

    match e {
        StreamError::StreamClosed => {
            std::process::exit(0x0100);
        }
        _ => (),
    }
}

#[derive(Error, Debug)]
pub enum ClientError<S: AsRef<str>> {
    #[error("\nNo path provided in command {0}\n    Usage: {0} <path>")]
    PathError(S),
}
/*
pub fn handle_file_error(e: Error) {
    eprintln!("Failed to handle file: {}", e)
}

pub fn handle_image_error(e: Error) {
    eprintln!("Failed to handle image: {}", e)
}

pub fn handle_text_error(e: Error) {
    eprintln!("Failed to handle text: {}", e)
}

pub fn no_path_error(command: &'static str) {
    eprintln!(
        "\nNo path provided in command {0}\n    Usage: {0} <path>",
        command
    );
}

pub fn write_stream_error(e: Error) {
    eprintln!("Write to stream failed: {}", e)
}
 */


#[derive(Serialize, Deserialize, Debug, Clone, Error)]
pub enum DBError {
    #[error("Failed to create DB pool")]
    PoolCreationError,
    #[error("Failed to get connection from pool")]
    ConnectionError,
    #[error("Failed to insert into users table")]
    UserInsertionError,
    #[error("Failed to insert into messages table")]
    MessageInsertionError,
    #[error("Message not found")]
    MessageNotFoundError,
    #[error("Failed to load messages, the database may not contain so many")]
    MessageHistoryError,
    #[error("User not found")]
    UserNotFoundError,
    #[error("Failed to verify password")]
    PasswordVerificationError,
}

impl DBError {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
    pub fn from_bytes(data: &[u8]) -> Self {
        bincode::deserialize(data).unwrap()
    }
}

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum ServerError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Unable to serialize object")]
    SerializeObjectError,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Username is used")]
    UsernameUsed,
}

impl ServerError {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
    pub fn from_bytes(data: &[u8]) -> Self {
        bincode::deserialize(data).unwrap()
    }
}
