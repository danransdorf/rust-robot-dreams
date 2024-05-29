use utils::get_address;
use client::start_client;

fn main() {
    let address = get_address();
    start_client(address)
}
