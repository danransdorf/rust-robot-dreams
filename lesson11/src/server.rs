use utils::get_address;
use server::start_server;

#[tokio::main]
async fn main() {
    let address = get_address();
    start_server(address).await
}
