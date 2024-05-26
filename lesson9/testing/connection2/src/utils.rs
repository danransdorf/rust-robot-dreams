use std::{
    fs::File,
    io::{ stdout, Error, ErrorKind, Read, Write },
    net::TcpStream,
    time::{ SystemTime, UNIX_EPOCH },
};

use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageData {
    Image(Vec<u8>),
    File(String, Vec<u8>),
    Text(String),
}
pub enum StreamError {
    ReadMessageError(Error),
    StreamClosed,
    FileCreationError(Error),
    FileWriteError(Error),
}

pub fn invalid_input_error(error: &'static str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, error)
}

pub fn handle_stream_error(e: StreamError) {
    match e {
        StreamError::ReadMessageError(e) => eprintln!("Failed to read message: {}", e),
        StreamError::StreamClosed => {
            eprintln!("Stream has been closed");
            std::process::exit(0x0100);
        }
        StreamError::FileCreationError(e) => eprintln!("Failed to create a file: {}", e),
        StreamError::FileWriteError(e) => eprintln!("Failed to create a file: {}", e),
    }
}

pub fn flush(message: &str) {
    writeln!(&mut stdout(), "{}", message).expect("Failed to write to output");
    stdout().flush().expect("Failed to flush output");
}

pub fn write_into_stream(mut stream: &TcpStream, content: &[u8]) {
    let len_bytes = (content.len() as u32).to_be_bytes();

    stream.write(&len_bytes).expect("Failed to write length bytes");
    stream.write_all(content).expect("Failed to write content");
}

pub fn serialize_data(data: MessageData) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(&data)
}

pub fn deserialize_data(data: Vec<u8>) -> Result<MessageData, bincode::Error> {
    bincode::deserialize(&data)
}

pub fn save_image(bytes: &Vec<u8>) -> Result<String, StreamError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let filename = format!("{:?}.png", timestamp);

    if let Err(e) = std::fs::create_dir_all("images") {
        return Err(StreamError::FileCreationError(e));
    }
    let file = File::create(&format!("images/{filename}"));

    match file {
        Err(e) => {
            return Err(StreamError::FileCreationError(e));
        }
        Ok(mut file) => {
            if let Err(e) = file.write_all(&bytes) {
                return Err(StreamError::FileWriteError(e));
            }
        }
    }

    Ok(filename)
}

pub fn save_file(filename: &str, bytes: &Vec<u8>) -> Result<String, StreamError> {
    if let Err(e) = std::fs::create_dir_all("files") {
        return Err(StreamError::FileCreationError(e));
    }
    let file_res = File::create(&format!("files/{filename}"));

    match file_res {
        Err(e) => {
            return Err(StreamError::FileCreationError(e));
        }
        Ok(mut file) => {
            if let Err(e) = file.write_all(&bytes) {
                return Err(StreamError::FileWriteError(e));
            }
        }
    }

    Ok(filename.to_string())
}

pub fn handle_stream(mut stream: &TcpStream) -> Result<MessageData, StreamError> {
    let mut len_buffer = [0; 4];

    if stream.read_exact(&mut len_buffer).is_err() {
        return Err(StreamError::StreamClosed);
    }

    let len = u32::from_be_bytes(len_buffer);
    let mut buffer = vec![0; len as usize];
    if let Err(e) = stream.read_exact(&mut buffer) {
        return Err(StreamError::ReadMessageError(e));
    }

    let message_data = deserialize_data(buffer).map_err(|_| {
        StreamError::ReadMessageError(Error::new(ErrorKind::InvalidData, "Failed to deserialize"))
    })?;
    match &message_data {
        MessageData::File(filename, bytes) => {
            let filename = save_file(filename, bytes)?;
            flush(&format!("Received file: {filename}"));
        }
        MessageData::Image(bytes) => {
            let filename = save_image(bytes)?;
            flush(&format!("Received image: {filename}"));
        }
        MessageData::Text(string) => {
            flush(&format!("Received message: {string}"));
        }
    }

    Ok(message_data)
}
