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