use std::{
    fs::File,
    io::{Cursor, Error, Read, Write},
    net::TcpStream,
    path::Path,
};

use image as image_crate;
use image_crate::ImageFormat;

use crate::{
    errors::invalid_input_error, file, image, message_request, serialize_stream, StreamRequest,
};
use crate::{utils::MessageContent, MessageRequest};

/// Get the image from the path and convert into a MessageContent
fn get_image(path: &Path) -> Result<MessageContent, Error> {
    let img = match image_crate::open(path) {
        Err(_) => {
            return Err(invalid_input_error("Failed to open image from path"));
        }
        Ok(x) => x,
    };

    let mut buf = Vec::new();
    if let Err(_) = img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png) {
        return Err(invalid_input_error(
            "Failed to convert image to .png, probably invalid/unknown image type",
        ));
    }

    Ok(image(buf))
}

/// Write the content into the stream
///
/// # Arguments
/// * `stream` - The stream to write into
/// * `content` - The content to write
fn write_into_stream(mut stream: &TcpStream, content: &[u8]) -> std::io::Result<()> {
    let len_bytes = (content.len() as u32).to_be_bytes();

    stream.write(&len_bytes)?;
    stream.write_all(content)?;

    Ok(())
}

/// Serialize the object and write it into the stream
///
/// # Arguments
/// * `stream` - The stream to write into
/// * `request` - The request object to serialize and write
pub fn serialize_and_write(stream: &TcpStream, request: StreamRequest) -> std::io::Result<()> {
    let serialized_string = match bincode::serialize(&request) {
        Ok(string) => string,
        _ => {
            return Err(invalid_input_error("Unable to serialize the request"));
        }
    };
    write_into_stream(stream, &serialized_string)
}

/// Handle the image at the path and write it into the stream
///
/// # Arguments
/// * `stream` - The stream to write into
/// * `path_string` - The path to the image
/// * `jwt` - The JWT auth token
pub fn handle_image(stream: &TcpStream, path_string: &str, jwt: String) -> std::io::Result<()> {
    let path = Path::new(path_string);

    let message = match get_image(path) {
        Ok(img) => img,
        Err(e) => {
            return Err(e);
        }
    };

    serialize_and_write(stream, message_request(MessageRequest::new(jwt, message)))
}

/// Handle the file at the path and write it into the stream
///
/// # Arguments
/// * `stream` - The stream to write into
/// * `path_string` - The path to the file
/// * `jwt` - The JWT auth token
pub fn handle_file(stream: &TcpStream, path_string: &str, jwt: String) -> std::io::Result<()> {
    let path = Path::new(path_string);

    let mut file_ref = File::open(path)?;
    let mut content = vec![];
    file_ref.read_to_end(&mut content)?;

    let message = file(
        path.file_name().unwrap().to_str().unwrap().to_string(),
        content,
    );

    serialize_and_write(stream, message_request(MessageRequest::new(jwt, message)))
}
