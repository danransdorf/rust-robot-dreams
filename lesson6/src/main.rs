use std::io::{ stdin, stdout, Write };
use std::io::Read;
use std::sync::mpsc::channel;
use std::thread;

mod operations;

const VALID_OPERATIONS: [&str; 5] = ["lowercase", "uppercase", "no-spaces", "slugify", "csv"];

fn detect_invalid_operation(operation: &str) -> bool {
    if !VALID_OPERATIONS.contains(&operation) {
        flush_print(
            &format!(
                "Operation `{}` isn't a possible operation. List of possible -- {}",
                operation,
                VALID_OPERATIONS.map(|string| format!("`{string}`")).join(",")
            )
        );
        true
    } else {
        false
    }
}

fn flush_print(message: &str) {
    println!("{}", message);
    stdout().flush().expect("Failed to flush stdout");
}

fn main() {
    let (tx, rx) = channel::<String>();
    let (continue_tx, continue_rx) = channel::<()>();

    continue_tx.send(()).unwrap();

    let continue_tx_clone = continue_tx.clone();

    let input_thread = thread::spawn(move || {
        while let Ok(_) = continue_rx.recv() {
            let mut operation = String::new();

            flush_print("Operation:");
            stdin()
                .read_line(&mut operation)
                .expect("Failed to read the input (line), that should represent an operation");

            operation = operation.trim().to_string();

            if detect_invalid_operation(&operation) {
                continue_tx_clone.send(()).unwrap();
                continue;
            }

            let mut input = String::new();

            flush_print("Input to process:");
            if operation == "csv" {
                flush_print("(Ctrl+D to end input)");
                stdin().read_to_string(&mut input).expect("Failed to read the input (whole)");
            } else {
                stdin().read_line(&mut input).expect("Failed to read the input (line)");
            }

            input = input.trim().to_string();

            tx.send(operation).unwrap();
            tx.send(input).unwrap();
        }
    });

    let processing_thread = thread::spawn(move || {
        loop {
            let operation = rx.recv().unwrap();
            let input = rx.recv().unwrap();

            let result = match operation.as_str() {
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

            continue_tx.send(()).unwrap();
        }
    });

    input_thread.join().unwrap();
    processing_thread.join().unwrap();
}
