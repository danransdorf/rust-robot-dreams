use utils::get_address;
use client::start_client;

#[tokio::main]
async fn main() {
    let address = get_address();
    start_client(address).await;
}
