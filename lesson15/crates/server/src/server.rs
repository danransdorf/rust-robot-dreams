use std::io::Error;
use std::{collections::HashMap, io::ErrorKind, net::SocketAddr, sync::Arc};
use tokio;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use utils::errors::{handle_stream_error, StreamError};
use utils::{deserialize_data, output_message_data, serialize_data, MessageData};

type WriteHalfArc = Arc<Mutex<WriteHalf<TcpStream>>>;
type StreamsHashMap = HashMap<SocketAddr, WriteHalfArc>;

use crate::db::DB;

pub async fn start_server(address: String) {
    let db = DB::new().unwrap();

    println!("Creating a server on address: {}", address);
    let listener = TcpListener::bind(address).await.unwrap();

    let clients: Arc<Mutex<StreamsHashMap>> = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (stream, client_addr) = listener.accept().await.unwrap();
        let (rd, wr) = io::split(stream);
        let reader = Arc::new(Mutex::new(rd));
        let writer = Arc::new(Mutex::new(wr));

        clients
            .lock()
            .await
            .insert(client_addr, Arc::clone(&writer));

        println!("Stream opened (addr: {})", client_addr);

        // Clients Arc::clone used in the spawned tokio
        let clients_clone = Arc::clone(&clients);
        tokio::spawn(async move {
            loop {
                match handle_stream(&reader).await {
                    Ok(message_data) => {
                        let serialized_string = match serialize_data(message_data) {
                            Ok(string) => string,
                            _ => {
                                eprintln!("Unable to serialize object");
                                continue;
                            }
                        };

                        for (addr, client_writer) in clients_clone.lock().await.iter() {
                            if *addr != client_addr {
                                let client_writer = client_writer.clone();
                                let serialized_string = serialized_string.clone();

                                tokio::spawn(async move {
                                    write_into_stream(&client_writer, &serialized_string)
                                        .await
                                        .map_err(|e| println!("{}", e))
                                        .ok();
                                });
                            }
                        }
                    }
                    Err(e) => match e {
                        StreamError::StreamClosed => {
                            eprintln!("Stream has been closed (addr: {})", &client_addr);
                            clients_clone.lock().await.remove(&client_addr);
                            break;
                        }
                        _ => handle_stream_error(e),
                    },
                }
            }
        });
    }
}

async fn write_into_stream(
    writer: &Arc<Mutex<WriteHalf<TcpStream>>>,
    content: &[u8],
) -> std::io::Result<()> {
    let len_bytes = (content.len() as u32).to_be_bytes();

    let mut locked_writer = writer.lock().await;
    locked_writer.write(&len_bytes).await?;
    locked_writer.write_all(content).await?;

    Ok(())
}

async fn handle_stream(
    reader: &Arc<Mutex<ReadHalf<TcpStream>>>,
) -> Result<MessageData, StreamError> {
    let mut locked_reader = reader.lock().await;

    let mut len_buffer = [0; 4];
    locked_reader
        .read_exact(&mut len_buffer)
        .await
        .map_err(|_| StreamError::StreamClosed)?;

    let len = u32::from_be_bytes(len_buffer);

    let mut buffer = vec![0; len as usize];
    locked_reader
        .read_exact(&mut buffer)
        .await
        .map_err(StreamError::ReadMessageError)?;

    drop(locked_reader); // Manually drop just to be sure

    let message_data = deserialize_data(buffer).map_err(|_| {
        StreamError::ReadMessageError(Error::new(ErrorKind::InvalidData, "Failed to deserialize"))
    })?;

    let message_data_clone = message_data.clone();
    tokio::spawn(async move {
        output_message_data(&message_data_clone);
    });

    Ok(message_data)
}
