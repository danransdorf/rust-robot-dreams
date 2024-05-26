use std::{ io::{ stdin, Read }, net::TcpStream };
use std::thread;

mod utils;
use utils::*;
mod write_utils;
use write_utils::*;

fn main() {
    let address = get_address();
    start(address)
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

fn start(address: String) {
    println!("Creating a client on address: {}", address);

    let stream = TcpStream::connect(&address).unwrap();
    let stream_clone = stream.try_clone().unwrap();

    thread::spawn(move || {
        loop {
            match handle_stream(&stream_clone).err() {
                Some(e) => handle_stream_error(e),
                _ => (),
            }
        }
    });

    thread
        ::spawn(move || {
            loop {
                flush("\nEnter message (Ctrl+D to send), send `.help` for possible commands:");

                let mut input_bytes = vec![];
                if let Err(e) = stdin().read_to_end(&mut input_bytes) {
                    flush("\n");
                    eprintln!("Failed to read the input: {}", e);
                    continue;
                }
                flush("\n");

                let input_string = String::from_utf8_lossy(&input_bytes).trim().to_string();
                let mut command = input_string.splitn(2, ' ');
                match command.next() {
                    Some(".file") => {
                        if let Some(path) = command.next() {
                            if let Err(e) = handle_file(&stream, path) {
                                eprintln!("Failed to handle file: {}", e);
                            }
                        }
                    }
                    Some(".image") => {
                        if let Some(path) = command.next() {
                            if let Err(e) = handle_image(&stream, path) {
                                eprintln!("Failed to handle image: {}", e);
                            }
                        }
                    }
                    Some(".quit") => exit_program(),
                    Some(".help") => print_help(),
                    _ => {
                        if
                            let Err(e) = serialize_and_write(
                                &stream,
                                MessageData::Text(input_string)
                            )
                        {
                            eprintln!("Failed to handle text: {}", e);
                        }
                    }
                }
            }
        })
        .join()
        .unwrap();
}
