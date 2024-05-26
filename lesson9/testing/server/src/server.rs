use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use crate::utils::*;

pub fn start(address: String) {
    println!("Creating a server on address: {}", address);

    let listener = TcpListener::bind(address).unwrap();

    let clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let client_addr = stream.peer_addr().unwrap();

        let clients_clone = Arc::clone(&clients);
        clients
            .lock()
            .unwrap()
            .insert(client_addr, stream.try_clone().unwrap());

        println!("Stream opened (addr: {})", client_addr);

        std::thread::spawn(move || loop {
            match handle_stream(&stream) {
                Ok(message_data) => {
                    let serialized_string = match serialize_data(message_data) {
                        Ok(string) => string,
                        _ => {
                            eprintln!("Unable to serialize object");
                            continue;
                        }
                    };
                    for (addr, client_stream) in clients_clone.lock().unwrap().iter() {
                        if *addr != client_addr {
                            write_into_stream(client_stream, &serialized_string);
                        }
                    }
                }
                Err(e) => match e {
                    StreamError::StreamClosed => {
                        eprintln!("Stream has been closed (addr: {})", &client_addr);
                        clients_clone.lock().unwrap().remove(&client_addr);
                        break;
                    }
                    _ => handle_stream_error(e),
                },
            }
        });
    }
}
