use client::start_client;
use utils::get_address;

#[tokio::main]
async fn main() {
    let address = get_address();
    start_client(address).await;
}
