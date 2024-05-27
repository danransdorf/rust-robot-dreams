use std::io::Error;

pub enum StreamError {
    ReadMessageError(Error),
    StreamClosed,
    FileCreationError(Error),
    FileWriteError(Error),
}

// This function is used in write_utils.rs, Rust just doesn't notice it I guess
#[allow(dead_code)]
pub fn invalid_input_error(error: &'static str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, error)
}

pub fn handle_stream_error(e: StreamError) {
    match e {
        StreamError::ReadMessageError(e) => eprintln!("Failed to read message: {}", e),
        StreamError::FileCreationError(e) => eprintln!("Failed to create a file: {}", e),
        StreamError::FileWriteError(e) => eprintln!("Failed to create a file: {}", e),
        StreamError::StreamClosed => {
            eprintln!("Stream has been closed");
            std::process::exit(0x0100);
        }
    }
}

// This function is used in client.rs, Rust just doesn't notice it I guess
#[allow(dead_code)]
pub fn handle_file_error(e: Error) {
    eprintln!("Failed to handle file: {}", e)
}

// This function is used in client.rs, Rust just doesn't notice it I guess
#[allow(dead_code)]
pub fn handle_image_error(e: Error) {
    eprintln!("Failed to handle image: {}", e)
}

// This function is used in client.rs, Rust just doesn't notice it I guess
#[allow(dead_code)]
pub fn handle_text_error(e: Error) {
    eprintln!("Failed to handle text: {}", e)
}

// This function is used in client.rs, Rust just doesn't notice it I guess
#[allow(dead_code)]
pub fn no_path_error(command: &'static str) {
    eprintln!("\nNo path provided in command {0}\n    Usage: {0} <path>", command);
}

// This function is used in server.rs, Rust just doesn't notice it I guess
#[allow(dead_code)]
pub fn write_stream_error(e: Error) {
    eprintln!("Write to stream failed: {}", e)
}
