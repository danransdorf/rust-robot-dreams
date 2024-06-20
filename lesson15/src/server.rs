use server::start_server;
use utils::get_address;

#[tokio::main]
async fn main() {
    let address = get_address();
    start_server(address).await
}
