use std::{
    fs::File,
    io::{Cursor, Error, Read, Write},
    net::TcpStream,
    path::Path,
};

use image::ImageFormat;

use crate::{errors::invalid_input_error, serialize_stream, StreamArrival};
use crate::{utils::MessageData, StreamMessage};

fn get_image(path: &Path) -> Result<MessageData, Error> {
    let img = match image::open(path) {
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

    Ok(MessageData::Image(buf))
}

fn write_into_stream(mut stream: &TcpStream, content: &[u8]) -> std::io::Result<()> {
    let len_bytes = (content.len() as u32).to_be_bytes();

    stream.write(&len_bytes)?;
    stream.write_all(content)?;

    Ok(())
}

pub fn serialize_and_write(stream: &TcpStream, obj: StreamArrival) -> std::io::Result<()> {
    let serialized_string = match serialize_stream(obj) {
        Ok(string) => string,
        _ => {
            return Err(invalid_input_error("Unable to serialize object"));
        }
    };
    write_into_stream(stream, &serialized_string)
}

pub fn handle_image(stream: &TcpStream, path_string: &str, jwt: String) -> std::io::Result<()> {
    let path = Path::new(path_string);

    let message = match get_image(path) {
        Ok(img) => img,
        Err(e) => {
            return Err(e);
        }
    };

    serialize_and_write(
        stream,
        StreamArrival::StreamMessage(StreamMessage::new(jwt, message)),
    )
}

pub fn handle_file(stream: &TcpStream, path_string: &str, jwt: String) -> std::io::Result<()> {
    let path = Path::new(path_string);

    let mut file = File::open(path)?;
    let mut content = vec![];
    file.read_to_end(&mut content)?;

    let message = MessageData::File(
        path.file_name().unwrap().to_str().unwrap().to_string(),
        content,
    );

    serialize_and_write(
        stream,
        StreamArrival::StreamMessage(StreamMessage::new(jwt, message)),
    )
}
