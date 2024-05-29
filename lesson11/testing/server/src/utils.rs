use std::{ fs::File, io::{ stdout, Error, ErrorKind, Read, Write }, net::TcpStream };

use serde::{ Deserialize, Serialize };
use chrono::Local;

pub mod errors;
use errors::{handle_stream_error, StreamError};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageData {
    Image(Vec<u8>),
    File(String, Vec<u8>),
    Text(String),
}


pub fn get_address() -> String {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        3 => format!("{}:{}", args.get(2).unwrap(), args.get(1).unwrap()),
        2 => format!("localhost:{}", args.get(1).unwrap()),
        1 => String::from("localhost:11111"),
        _ => {
            panic!(
                "Please specify `client` or `server` using command line arguments. Command line arguments expected: <client/server> <port> <hostname>"
            )
        }
    }
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

static SECONDS_INDEX: usize = 19;
pub fn save_image(bytes: &Vec<u8>) -> Result<String, StreamError> {
    let timestamp = &Local::now().to_string()[..SECONDS_INDEX];
    let filename = format!("{}.png", timestamp);

    std::fs::create_dir_all("images").map_err(StreamError::FileCreationError)?;

    let mut file = File::create(&format!("images/{filename}")).map_err(
        StreamError::FileCreationError
    )?;
    file.write_all(&bytes).map_err(StreamError::FileWriteError)?;

    Ok(filename)
}

pub fn save_file(filename: &str, bytes: &Vec<u8>) -> Result<String, StreamError> {
    std::fs::create_dir_all("files").map_err(StreamError::FileCreationError)?;

    let mut file = File::create(&format!("files/{filename}")).map_err(
        StreamError::FileCreationError
    )?;
    file.write_all(&bytes).map_err(StreamError::FileWriteError)?;

    Ok(filename.to_string())
}

pub fn output_message_data(message_data: &MessageData) {
    match message_data {
        MessageData::File(filename, bytes) => {
            match save_file(filename, bytes) {
                Ok(filename) => flush(&format!("Received file: {filename}")),
                Err(e) => {
                    flush("Received file, but failed to save it");
                    handle_stream_error(e)
                }
            };
        }
        MessageData::Image(bytes) => {
            match save_image(bytes) {
                Ok(filename) => flush(&format!("Received image: {filename}")),
                Err(e) => {
                    flush("Received image, but failed to save it");
                    handle_stream_error(e)
                }
            };
        }
        MessageData::Text(string) => {
            flush(&format!("Received message: {string}"));
        }
    }
}

// This function is used in client.rs, Rust just doesn't notice it I guess
#[allow(dead_code)]
pub fn handle_stream(mut stream: &TcpStream) -> Result<MessageData, StreamError> {
    let mut len_buffer = [0; 4];

    stream.read_exact(&mut len_buffer).map_err(|_| StreamError::StreamClosed)?;

    let len = u32::from_be_bytes(len_buffer);
    let mut buffer = vec![0; len as usize];
    stream.read_exact(&mut buffer).map_err(StreamError::ReadMessageError)?;

    let message_data = deserialize_data(buffer).map_err(|_| {
        StreamError::ReadMessageError(Error::new(ErrorKind::InvalidData, "Failed to deserialize"))
    })?;
    output_message_data(&message_data);

    Ok(message_data)
}
