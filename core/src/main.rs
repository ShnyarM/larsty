use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Please provide a file to compile.");
    }

    let file_path = &args[1];
    let program = fs::read_to_string(file_path)
        .expect("Error reading the given file.");

    core::compile(&program);
}

