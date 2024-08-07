use std::{
    fs::File,
    io::{stdout, Write},
    sync::Arc,
};

use crate::db::{structs::Message, DB};
use anyhow::Result;
use chrono::Local;
use clap::{arg, command, Parser};

use crate::errors::{deserialize_object_error, handle_stream_error, StreamError};

mod structs;
pub use structs::*;

impl ServerResponse {
    pub fn serialize(self) -> String {
        serialize_server_response(self).unwrap()
    }
}

impl MessageResponse {
    pub fn from_db_message(message: &Message, db: &Arc<DB>) -> Result<Self, ErrorResponse> {
        let user = db.get_user(message.user_id).map_err(|e| db_error(e))?;
        let content = deserialize_data(message.content.to_owned())
            .map_err(|_| server_error(deserialize_object_error()))?;
        Ok(MessageResponse {
            id: message.id.unwrap(),
            username: user.username,
            user_id: message.user_id,
            content,
        })
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "localhost")]
    pub hostname: String,

    #[arg(long, default_value = "11111")]
    pub port: String,
}

pub fn get_address() -> String {
    let args = Args::parse();

    format!("{}:{}", args.hostname, args.port)
}

pub fn flush(message: &str) {
    writeln!(&mut stdout(), "{}", message).expect("Failed to write to output");
    stdout().flush().expect("Failed to flush output");
}

pub fn serialize_data(data: MessageContent) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(&data)
}
pub fn deserialize_data(data: Vec<u8>) -> Result<MessageContent, bincode::Error> {
    bincode::deserialize(&data)
}

pub fn serialize_server_response(data: ServerResponse) -> Result<String, serde_json::Error> {
    serde_json::to_string(&data)
}
pub fn deserialize_server_response(data: String) -> Result<ServerResponse, serde_json::Error> {
    serde_json::from_str(&data)
}

pub fn serialize_stream(request: StreamRequest) -> Result<String, serde_json::Error> {
    serde_json::to_string(&request)
}
pub fn deserialize_stream(string: String) -> Result<StreamRequest, serde_json::Error> {
    serde_json::from_str(&string)
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
        MessageContent::File(filename, bytes) => {
            match save_file(&filename, &bytes) {
                Ok(filename) => flush(&format!(
                    "{}: sent a file {}",
                    message_data.username, filename
                )),
                Err(e) => {
                    flush("Received file, but failed to save it");
                    handle_stream_error(e)
                }
            };
        }
        MessageContent::Image(bytes) => {
            match save_image(&bytes) {
                Ok(filename) => flush(&format!(
                    "{}: sent an image {}",
                    message_data.username, filename
                )),
                Err(e) => {
                    flush("Received image, but failed to save it");
                    handle_stream_error(e)
                }
            };
        }
        MessageContent::Text(string) => {
            flush(&format!("{}: {}", message_data.username, string));
        }
    }
}

pub fn unspecified_error() -> Result<(), std::io::Error> {
    return Err(std::io::Error::new(std::io::ErrorKind::Other, ""));
}
