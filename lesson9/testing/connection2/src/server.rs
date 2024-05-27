use std::io::Error;
use std::{ collections::HashMap, io::ErrorKind, net::SocketAddr, sync::Arc };
use tokio;
use tokio::net::{ TcpListener, TcpStream };
use tokio::sync::Mutex;
use tokio::io::{ AsyncReadExt, AsyncWriteExt };

mod utils;
use utils::errors::{ handle_stream_error, write_stream_error, StreamError };
use utils::{ deserialize_data, get_address, output_message_data, serialize_data, MessageData };

#[tokio::main]
async fn main() {
    let address = get_address();
    start_server(address).await
}

type StreamArc = Arc<Mutex<TcpStream>>;
type StreamsHashMap = HashMap<SocketAddr, StreamArc>;

async fn start_server(address: String) {
    println!("Creating a server on address: {}", address);

    let listener = TcpListener::bind(address).await.unwrap();

    let clients: Arc<Mutex<StreamsHashMap>> = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (stream, client_addr) = listener.accept().await.unwrap();
        let stream = Arc::new(Mutex::new(stream));

        clients.lock().await.insert(client_addr, Arc::clone(&stream));

        println!("Stream opened (addr: {})", client_addr);

        // Clients Arc::clone used in the spawned tokio
        let clients_clone = Arc::clone(&clients);
        tokio::spawn(async move {
            loop {
                match handle_stream(&stream).await {
                    Ok(message_data) => {
                        let serialized_string = match serialize_data(message_data) {
                            Ok(string) => string,
                            _ => {
                                eprintln!("Unable to serialize object");
                                continue;
                            }
                        };

                        for (addr, client_stream) in clients_clone.lock().await.iter() {
                            if *addr != client_addr {
                                let client_stream = client_stream.clone();
                                let serialized_string = serialized_string.clone();
                                tokio::spawn(async move {
                                    write_into_stream(&client_stream, &serialized_string).await
                                        .map_err(write_stream_error)
                                        .ok()
                                });
                            }
                        }
                    }
                    Err(e) =>
                        match e {
                            StreamError::StreamClosed => {
                                eprintln!("Stream has been closed (addr: {})", &client_addr);
                                clients_clone.lock().await.remove(&client_addr);
                                break;
                            }
                            _ => handle_stream_error(e),
                        }
                }
            }
        });
    }
}

async fn write_into_stream(stream: &Arc<Mutex<TcpStream>>, content: &[u8]) -> std::io::Result<()> {
    let len_bytes = (content.len() as u32).to_be_bytes();

    let mut locked_stream = stream.lock().await;
    locked_stream.write(&len_bytes).await?;
    locked_stream.write_all(content).await?;

    Ok(())
}

async fn handle_stream(stream: &Arc<Mutex<TcpStream>>) -> Result<MessageData, StreamError> {
    let mut locked_stream = stream.lock().await;

    let mut len_buffer = [0; 4];
    locked_stream.read_exact(&mut len_buffer).await.map_err(|_| StreamError::StreamClosed)?;

    let len = u32::from_be_bytes(len_buffer);

    let mut buffer = vec![0; len as usize];
    locked_stream.read_exact(&mut buffer).await.map_err(StreamError::ReadMessageError)?;

    drop(locked_stream); // Manually drop just to be sure

    let message_data = deserialize_data(buffer).map_err(|_| {
        StreamError::ReadMessageError(Error::new(ErrorKind::InvalidData, "Failed to deserialize"))
    })?;

    let message_data_clone = message_data.clone();
    tokio::spawn(async move {
        output_message_data(&message_data_clone);
    });

    Ok(message_data)
}
