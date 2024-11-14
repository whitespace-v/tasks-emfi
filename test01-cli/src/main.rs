#![warn(clippy::all, clippy::pedantic)]
use std::{
    env,
    fs::File,
    io::{BufReader, Read},
};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let file_path = match args.get(1) {
        Some(path) => path,
        None => {
            eprintln!("Error: No file path provided.");
            std::process::exit(1);
        }
    };

    let file = match File::open(file_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "Error: Failed to open file '{}': \nSTDERR: {}",
                file_path, e
            );
            std::process::exit(1);
        }
    };

    let mut word_count = 0;
    let mut line_count = 0;
    let mut character_count = 0;

    let mut buffer = String::new();

    match BufReader::new(file).read_to_string(&mut buffer) {
        Ok(0) => {
            eprintln!("Error: The file is empty or could not be read properly.");
            std::process::exit(1);
        }
        Ok(_) => {
            character_count += buffer.chars().count();
            line_count += buffer.lines().count();
            word_count += buffer.split_whitespace().count();
        }
        Err(e) => {
            eprintln!("Error: Failed to read from file '{}': {}", file_path, e);
            std::process::exit(1); // Exit with an error status
        }
    }

    println!(
        "Words: {}\nLines: {}\nCharacters: {}",
        word_count, line_count, character_count
    );
    Ok(())
}