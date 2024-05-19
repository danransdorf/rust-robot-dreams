use std::io::{ stdin, stdout, Write };
use std::io::Read;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;

mod operations;

enum Operation {
    Lowercase,
    Uppercase,
    NoSpaces,
    Slugify,
    Csv,
}

#[derive(Debug)]
enum OperationError {
    InvalidOperation(String),
}

impl FromStr for Operation {
    type Err = OperationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "lowercase" => Ok(Operation::Lowercase),
            "uppercase" => Ok(Operation::Uppercase),
            "no-spaces" => Ok(Operation::NoSpaces),
            "slugify" => Ok(Operation::Slugify),
            "csv" => Ok(Operation::Csv),
            _ => Err(OperationError::InvalidOperation(s.to_string())),
        }
    }
}

const VALID_OPERATIONS: [&str; 5] = ["lowercase", "uppercase", "no-spaces", "slugify", "csv"];

fn flush_eprint(message: &str) {
    eprintln!("{}", message);
    stdout().flush().expect("Failed to flush stdout");
}
fn flush_print(message: &str) {
    println!("{}", message);
    stdout().flush().expect("Failed to flush stdout");
}

fn main() {
    let (operation_tx, operation_rx) = channel::<Operation>();
    let (input_tx, input_rx) = channel::<String>();
    let (continue_tx, continue_rx) = channel::<()>();

    continue_tx.send(()).unwrap();

    let continue_tx_clone = continue_tx.clone();

    let input_thread = thread::spawn(move || {
        while let Ok(_) = continue_rx.recv() {
            let mut input_buf = String::new();

            flush_print("Enter <operation>⎵<input> (Ctrl+D to submit):");
            stdin().read_to_string(&mut input_buf).expect("Failed to read the input");

            if let Some((operation, input)) = input_buf.split_once(" ") {
                match Operation::from_str(operation.trim()) {
                    Ok(operation) => {
                        operation_tx.send(operation).unwrap();
                        input_tx.send(input.trim().to_string()).unwrap();
                    }
                    Err(OperationError::InvalidOperation(op)) => {
                        flush_eprint(
                            &format!(
                                "Operation `{}` isn't a possible operation. List of possible -- {}",
                                op,
                                VALID_OPERATIONS.join(", ")
                            )
                        );
                        continue_tx_clone.send(()).unwrap();
                        continue;
                    }
                }
            } else {
                flush_eprint("No input provided, use format: <operation>⎵<input> \n");
                continue_tx_clone.send(()).unwrap();
                continue;
            }
        }
    });

    let processing_thread = thread::spawn(move || {
        loop {
            let operation = operation_rx.recv().unwrap();
            let input = input_rx.recv().unwrap();

            let result = match operation {
                Operation::Lowercase => operations::lowercase(input),
                Operation::Uppercase => operations::uppercase(input),
                Operation::NoSpaces => operations::no_spaces(input),
                Operation::Slugify => operations::slugify(input),
                Operation::Csv => operations::csv(input),
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
