use std::error::Error;
use std::str::FromStr;

mod csv;

pub const VALID_OPERATIONS: [&str; 5] = ["lowercase", "uppercase", "no-spaces", "slugify", "csv"];

#[derive(Debug)]
pub enum OperationError {
    InvalidOperation(String),
}

pub enum Operation {
    Lowercase,
    Uppercase,
    NoSpaces,
    Slugify,
    Csv,
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

pub fn fmt_invalid_op_error(op: String) -> String {
    format!(
        "Operation `{}` isn't a possible operation. List of possible -- {}",
        op,
        VALID_OPERATIONS.join(", ")
    )
}

pub fn execute_operation(operation: Operation, input: String) -> Result<String, Box<dyn Error>> {
    match operation {
        Operation::Lowercase => lowercase(input),
        Operation::Uppercase => uppercase(input),
        Operation::NoSpaces => no_spaces(input),
        Operation::Slugify => slugify(input),
        Operation::Csv => csv(input),
    }
}

pub fn lowercase(input: String) -> Result<String, Box<dyn Error>> {
    Ok(input.to_lowercase())
}

pub fn uppercase(input: String) -> Result<String, Box<dyn Error>> {
    Ok(input.to_uppercase())
}

pub fn no_spaces(input: String) -> Result<String, Box<dyn Error>> {
    Ok(input.replace(" ", ""))
}

pub fn slugify(input: String) -> Result<String, Box<dyn Error>> {
    Ok(slug::slugify(input))
}

pub fn csv(input: String) -> Result<String, Box<dyn Error>> {
    let csv = csv::Csv::from(&input);

    Ok(csv?.to_string())
}
