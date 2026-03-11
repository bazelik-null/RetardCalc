pub mod interpreter;

use interpreter::tokens;
use std::io;
use std::io::Write;

fn main() {
    println!("=====================================================");
    println!("==== RetardCalc: Worst math expression evaluator ====");
    println!("=====================================================");

    loop {
        print!(">>> "); // Input indicator
        io::stdout().flush().unwrap(); // Ensure prompt displays immediately

        let mut input: String = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(..) => {
                let trimmed = input.trim();

                // Skip empty lines
                if trimmed.is_empty() {
                    continue;
                }
                // Exit loop
                if trimmed == "exit" || trimmed == "quit" {
                    break;
                }

                calculate(trimmed.to_string());
            }
            Err(error) => println!("[ERROR]: {error}"),
        }
    }
}

fn calculate(input: String) {
    // Parse expression
    let tokens: Vec<tokens::Token> = match interpreter::lexer::tokenize(input) {
        Some(tokens) => tokens,
        None => {
            eprintln!("[ERROR]: No tokens found");
            return;
        }
    };

    // Evaluate expression
    let eval: f64 = match interpreter::evaluator::eval(&tokens) {
        Some(eval) => eval,
        None => {
            eprintln!("[ERROR]: Invalid expression");
            return;
        }
    };
    println!("{}", eval);
}
