use slug::slugify;
use std::env;
use std::io;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        panic!("No parameter passed")
    }

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input.");

    if args[1].as_str() == "lowercase" {
        println!("{}", input.to_lowercase())
    } else if args[1].as_str() == "uppercase" {
        println!("{}", input.to_uppercase())
    } else if args[1].as_str() == "no-spaces" {
        println!("{}", input.replace(" ", ""))
    } else if args[1].as_str() == "slugify" {
        println!("{}", slugify(input))
    } else {
        panic!("Invalid parameter passed, `{}` isn't contained in -- `lowercase`,`uppercase`,`no-spaces`,`slugify`", args[1])
    }
}
