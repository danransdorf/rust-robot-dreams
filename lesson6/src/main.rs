use std::io::{ stderr, stdin, stdout, Write };
use std::io::Read;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;

use operations::{ execute_operation, fmt_invalid_op_error, Operation, OperationError };

mod operations;

fn flush<W: Write>(message: &str, writer: &mut W) {
    writeln!(writer, "{}", message).expect("Failed to write to output");
    writer.flush().expect("Failed to flush output");
}

fn main() {
    // Transmitting selected operation, input_thread => processing_thread
    let (operation_tx, operation_rx) = channel::<Operation>();
    // Transmitting the string to process, input_thread => processing_thread
    let (input_tx, input_rx) = channel::<String>();
    // Controls the flow of input_thread. Receive message => trigger next round of reading user input
    let (next_round_tx, next_round_rx) = channel::<()>();

    let next_round_tx_clone = next_round_tx.clone();

    // Enqueue first round of reading user input
    next_round_tx.send(()).unwrap();
    let input_thread = thread::spawn(move || {
        // Await message => trigger next round
        while let Ok(_) = next_round_rx.recv() {
            flush("Enter <operation>⎵<input> (Ctrl+D to submit):", &mut stdout());
            // Read stdin
            let mut input_buf = String::new();
            stdin().read_to_string(&mut input_buf).expect("Failed to read the input");
            // Parse `<operation>⎵<input>` format
            if let Some((operation, input)) = input_buf.split_once(" ") {
                match Operation::from_str(operation.trim()) {
                    // Valid operation => send to processing thread
                    Ok(operation) => {
                        operation_tx.send(operation).unwrap();
                        input_tx.send(input.trim().to_string()).unwrap();
                    }
                    // Invalid Operation => print error, next input round
                    Err(OperationError::InvalidOperation(op)) => {
                        flush(&fmt_invalid_op_error(op), &mut stderr());
                        next_round_tx_clone.send(()).unwrap();
                        continue;
                    }
                }
            } else {
                // Input not in `<operation>⎵<input>` format => print error, next input round
                flush("No input provided, use format: <operation>⎵<input> \n", &mut stderr());
                next_round_tx_clone.send(()).unwrap();
                continue;
            }
        }
    });

    let processing_thread = thread::spawn(move || {
        loop {
            // Receive user's selected operation and input from input_thread
            let operation = operation_rx.recv().unwrap();
            let input = input_rx.recv().unwrap();
            // Execute operation on the input
            let result = execute_operation(operation, input);
            // Output
            match result {
                Ok(outcome) => println!("{}", outcome),
                Err(e) => eprintln!("Error: {}", e),
            }
            // Trigger next round
            next_round_tx.send(()).unwrap();
        }
    });

    input_thread.join().unwrap();
    processing_thread.join().unwrap();
}
