use std::{ io::{ stdin, Read }, net::TcpStream };
use std::thread;

mod utils;
use utils::{
    errors::{
        handle_file_error,
        handle_image_error,
        handle_stream_error,
        handle_text_error,
        no_path_error,
    },
    flush,
    get_address,
    handle_stream,
    MessageData,
};
mod write_utils;
use write_utils::*;

fn main() {
    let address = get_address();
    start_client(address)
}

fn print_help() {
    println!("Possible commands:");
    println!("    .file <path>");
    println!("    .image <path>");
    println!("    .quit");
    println!("    .help");
}
fn exit_program() {
    println!("Exiting program...");
    std::process::exit(0x0100);
}

fn start_client(address: String) {
    println!("Creating a client on address: {}", address);

    let stream = TcpStream::connect(&address).unwrap();
    let stream_clone = stream.try_clone().unwrap();

    thread::spawn(move || {
        loop {
            handle_stream(&stream_clone).map_err(handle_stream_error).ok();
        }
    });

    thread
        ::spawn(move || {
            loop {
                flush("\nEnter message (Ctrl+D to send), send `.help` for possible commands:");

                let mut input_bytes = vec![];
                if let Err(e) = stdin().read_to_end(&mut input_bytes) {
                    eprintln!("\nFailed to read the input: {}", e);
                    continue;
                }
                flush("\n");

                let input_string = String::from_utf8_lossy(&input_bytes).trim().to_string();
                let mut command = input_string.splitn(2, ' ');
                match command.next() {
                    Some(".quit") => exit_program(),
                    Some(".help") => print_help(),
                    Some(".image") => {
                        if let Some(path) = command.next() {
                            handle_image(&stream, path).map_err(handle_image_error).ok();
                        } else {
                            no_path_error(".image");
                        }
                    }
                    Some(".file") => {
                        if let Some(path) = command.next() {
                            handle_file(&stream, path).map_err(handle_file_error).ok();
                        } else {
                            no_path_error(".file");
                        }
                    }
                    _ => {
                        serialize_and_write(&stream, MessageData::Text(input_string))
                            .map_err(handle_text_error)
                            .ok();
                    }
                }
            }
        })
        .join()
        .unwrap();
}
