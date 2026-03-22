use crate::core::compiler::preprocessor::lexer::Lexer;
use std::{env, fs, io};

pub mod core;

fn main() {
    let args: Vec<String> = env::args().collect();

    // If filepath passed execute file.
    if args.len() > 1 {
        if let Err(err) = execute_file(&args[1]) {
            eprintln!("[ERROR]: {}", err);
            std::process::exit(1);
        }
    }
    // Else enter CLI
    else {
        unimplemented!()
    }
}

pub fn execute_file(file_path: &str) -> io::Result<()> {
    let input = fs::read_to_string(file_path)?;
    execute(input.trim(), file_path);
    Ok(())
}

fn execute(input: &str, filename: &str) {
    // Tokenize input
    let lexer = Lexer::new(input.parse().unwrap(), filename);
    let result = lexer.scan();
    if !result.errors.is_empty() {
        for error in result.errors {
            eprintln!("{}", error);
        }
        return;
    }

    // Just print tokens for now
    for token in result.tokens {
        print!("{:?} ", token.token_type)
    }
}
