use client::run_app;
use utils::get_address;

fn main() {
    let address = get_address();
    run_app(address)
}
