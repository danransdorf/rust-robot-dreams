mod client;
mod server;
mod utils;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let address = match args.len() {
        4 => format!("{}:{}", args.get(3).unwrap(), args.get(2).unwrap()),
        3 => format!("localhost:{}", args.get(2).unwrap()),
        2 => String::from("localhost:11111"),
        _ => {
            panic!(
                "Please specify `client` or `server` using command line arguments. Command line arguments expected: <client/server> <port> <hostname>"
            )
        }
    };

    match args[1].as_str() {
        "server" => server::start(address),
        "client" => client::start(address),
        _ =>
            println!(
                "Please specify `client` or `server` using command line arguments. Command line arguments expected: <client/server> <port> <hostname>"
            ),
    }
}
