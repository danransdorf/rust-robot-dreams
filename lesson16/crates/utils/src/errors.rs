use serde::{Deserialize, Serialize};
use std::io::Error;
use thiserror::Error;

use init_macros::create_enum_init_functions;

/// Error type for stream-related issues
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

/// Create a new error for invalid input
pub fn invalid_input_error(error: &'static str) -> Error {
    Error::new(std::io::ErrorKind::InvalidInput, error)
}

/// Handle a stream error
pub fn handle_stream_error(e: StreamError) {
    eprintln!("{}", e);

    match e {
        StreamError::StreamClosed => {
            std::process::exit(0x0100);
        }
        _ => (),
    }
}

/// Error type for client-related issues
#[derive(Error, Debug)]
pub enum ClientError<S: AsRef<str>> {
    #[error("\nNo path provided in command {0}\n    Usage: {0} <path>")]
    PathError(S),
}

/// Error type for database-related issues
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
    /// Serialize a DBError into bytes
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
    /// Deserialize bytes into a DBError
    pub fn from_bytes(data: &[u8]) -> Self {
        bincode::deserialize(data).unwrap()
    }
}

/// Error type for server-related issues
#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum ServerError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Unable to serialize object")]
    SerializeObjectError,
    #[error("Unable to deserialize object")]
    DeserializeObjectError,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Username is used")]
    UsernameUsed,
}

impl ServerError {
    /// Serialize a ServerError into bytes
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
    /// Deserialize bytes into a ServerError
    pub fn from_bytes(data: &[u8]) -> Self {
        bincode::deserialize(data).unwrap()
    }
}

create_enum_init_functions!(
    DBError,
    PoolCreationError,
    ConnectionError,
    UserInsertionError,
    MessageInsertionError,
    MessageNotFoundError,
    MessageHistoryError,
    UserNotFoundError,
    PasswordVerificationError
);
create_enum_init_functions!(
    ServerError,
    InvalidToken,
    SerializeObjectError,
    DeserializeObjectError,
    InvalidCredentials,
    UsernameUsed
);
