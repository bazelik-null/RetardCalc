// Copyright (c) 2026 bazelik-null

use crate::cli::app_state::AppState;
use crate::cli::backend::{calculate_with_result, eval_file};
use crate::cli::cmd::Command;

use std::io::{self, Write};

pub fn cli_init() {
    print_banner();
    let mut state = AppState::default();
    let mut input_buffer = String::new();

    loop {
        if !handle_input_cycle(&mut state, &mut input_buffer) {
            break;
        }
    }

    println!("[INFO]: Exiting...");
}

// Input handling

fn handle_input_cycle(state: &mut AppState, input_buffer: &mut String) -> bool {
    print_prompt();

    input_buffer.clear();
    if let Err(e) = io::stdin().read_line(input_buffer) {
        eprintln!("[ERROR]: {}", e);
        return true; // Continue on IO error
    }

    let trimmed = input_buffer.trim();

    // Skip empty lines
    if trimmed.is_empty() {
        return true;
    }

    // Check for commands
    match Command::from_input(trimmed) {
        Command::Exit => return false,
        Command::Debug => {
            state.toggle_debug();
            return true;
        }
        Command::Help => {
            print_help();
            return true;
        }
        Command::Func => {
            print_func();
            return true;
        }
        Command::File(file_path) => {
            match eval_file(file_path.as_str(), state.is_debug) {
                Ok(_) => {} // eval_file prints result.
                Err(err) => eprintln!("[ERROR]: {}", err),
            }
            return true;
        }
        Command::Unknown => {} // Continue to calculation
    }

    // Calculate and display result
    match calculate_with_result(trimmed, state.is_debug) {
        Ok(result) => println!("{}", result),
        Err(err) => eprintln!("[ERROR]: {}", err),
    }

    true
}

// UI

fn print_banner() {
    println!("======================================");
    println!("==== Morsel Interpreter Interface ====");
    println!("======================================");
    println!();
}

fn print_prompt() {
    print!(">>> ");
    io::stdout().flush().expect("Failed to flush stdout");
}

fn print_help() {
    println!("Available commands\n");
    println!("  help     - Show this help message.");
    println!("  func     - Show all math functions.");
    println!("  file     - Evaluate passed file.");
    println!("  debug    - Toggle debug mode.");
    println!("  exit     - Exit.\n");

    println!("Enter any mathematical expression to evaluate it.\n");
}

fn print_func() {
    println!("Available Functions:\n");

    println!("Arithmetic Operations:");
    println!("  Addition:       x + y");
    println!("  Subtraction:    x - y");
    println!("  Multiplication: x * y");
    println!("  Division:       x / y\n");

    println!("Exponent and Logarithmic Operations:");
    println!("  Exponentiation:    x ^ y");
    println!("  Square root:       sqrt(x)");
    println!("  Logarithm:         log(x, y) [where x is base, y is argument]");
    println!("  Natural logarithm: ln(x)\n");

    println!("Trigonometric Functions:");
    println!("  Cosine:     cos(x)");
    println!("  Sine:       sin(x)");
    println!("  Tangent:    tan(x)");
    println!("  Arccosine:  acos(x)");
    println!("  Arcsine:    asin(x)");
    println!("  Arctangent: atan(x)\n");

    println!("Miscellaneous Operations:");
    println!("  Negation:           -x");
    println!("  Modulo (remainder): x % y");
    println!("  Absolute value:     abs(x)");
    println!("  Rounding:           round(x)");
    println!("  Max value:          max(x, ...)");
    println!("  Min value:          max(x, ...)\n");

    println!("  Unknown: Default value [invalid]\n");
}
