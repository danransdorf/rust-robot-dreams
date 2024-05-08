use std::env;
use std::io;
use std::io::Read;

mod operations;

const VALID_OPERATIONS: [&str; 5] = ["lowercase", "uppercase", "no-spaces", "slugify", "csv"];

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        panic!("No parameter passed");
    }

    let operation = args[1].as_str();

    assert!(
        VALID_OPERATIONS.contains(&operation),
        "Operation `{}` isn't a possible operation. List of possible -- {}",
        operation,
        VALID_OPERATIONS.map(|string| format!("`{string}`")).join(",")
    );

    let mut input = String::new();

    match operation {
        "csv" => {
            match io::stdin().read_to_string(&mut input) {
                Err(e) => {
                    eprintln!("Failed to read the input (whole): {}", e);
                }
                _ => (),
            }
        }
        _ => {
            match io::stdin().read_line(&mut input) {
                Err(e) => {
                    eprintln!("Failed to read the input (line): {}", e);
                    return;
                }
                _ => (),
            }
        }
    }

    let result = match operation {
        "lowercase" => operations::lowercase(input),
        "uppercase" => operations::uppercase(input),
        "no-spaces" => operations::no_spaces(input),
        "slugify" => operations::slugify(input),
        "csv" => operations::csv(input),
        _ => unreachable!(),
    };

    match result {
        Ok(outcome) => println!("{}", outcome),
        Err(e) => eprintln!("Error: {}", e),
    }
}
