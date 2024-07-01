use futures::stream::StreamExt;
use tokio::net::{TcpListener};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::accept_async;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub async fn start_server(address: String) {
    let listener = TcpListener::bind(address).await.unwrap();
    println!("WebSocket server running");

    let clients: Arc<Mutex<HashMap<_, _>>> = Arc::new(Mutex::new(HashMap::new()));

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, clients.clone()));
    }
}

async fn handle_connection(raw_stream: tokio::net::TcpStream, clients: Arc<Mutex<HashMap<_, _>>>) {
    let ws_stream = accept_async(raw_stream).await.expect("Error during the websocket handshake");

    let (write, read) = ws_stream.split();

    // Example: Echo incoming WebSocket messages back to the client
    read.for_each(|message| async {
        let message = message.unwrap();

        if let Message::Text(text) = message {
            // Here you can handle the message, e.g., by echoing it back
            write.send(Message::Text(text)).await.unwrap();
        }
    }).await;
}