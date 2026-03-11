pub mod interpreter;

use interpreter::tokens;
use std::error::Error;
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

                let code = calculate(trimmed.to_string());
                match code {
                    Ok(result) => {
                        println!("{}", result)
                    }
                    Err(err) => {
                        eprintln!("[ERROR]: {}", err)
                    }
                }
            }
            Err(error) => println!("[ERROR]: {error}"),
        }
    }
}

fn calculate(input: String) -> Result<f64, Box<dyn Error>> {
    // Parse expression
    let tokens: Vec<tokens::Token> = interpreter::lexer::tokenize(input)?;

    // Evaluate expression
    let eval: f64 = interpreter::evaluator::eval(&tokens)?;

    Ok(eval)
}
