use thiserror::Error;

#[derive(Error, Debug)]
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
