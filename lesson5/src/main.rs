use std::env;
use std::io;
use std::io::Read;

mod operations;

const VALID_OPERATIONS: [&str; 5] = ["lowercase", "uppercase", "no-spaces", "slugify", "csv"];

fn check_operation_validity(operation: &str) {
    assert!(
        VALID_OPERATIONS.contains(&operation),
        "Operation `{}` isn't a possible operation. List of possible -- {}",
        operation,
        VALID_OPERATIONS.map(|string| format!("`{string}`")).join(",")
    )
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        eprintln!("No parameter passed, pass a parameter e.g. cargo run -- lowercase");
        eprintln!(
            "Operations: {}",
            VALID_OPERATIONS.iter()
                .map(|operation| format!("`{operation}`"))
                .collect::<Vec<String>>()
                .join(",")
        );
        std::process::exit(1);
    }

    let operation = args[1].as_str();

    check_operation_validity(&operation);

    let mut input = String::new();

    if operation == "csv" {
        io::stdin().read_to_string(&mut input).expect("Failed to read the input (whole)");
    } else {
        io::stdin().read_line(&mut input).expect("Failed to read the input (line)");
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
