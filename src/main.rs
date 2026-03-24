use crate::core::compiler::error_handler::CompilerError;
use crate::core::compiler::parser::parser::Parser;
use crate::core::compiler::preprocessor::lexer::Lexer;
use crate::core::compiler::source::SourceCode;
use colored::Colorize;
use lasso::Rodeo;
use std::{env, fs, io};

pub mod core;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        if let Err(err) = execute_file(&args[1]) {
            eprintln!("[ERROR]: {}", err);
            std::process::exit(1);
        }
    } else {
        start_interactive_cli();
    }
}

fn start_interactive_cli() {
    println!("{}", "=== Morsel CLI ===".cyan().bold());
    println!("{}", "Commands:".cyan());
    println!("  {} - Execute code", "'code'".yellow());
    println!("  {} - Load and execute file", "'load <path>'".yellow());
    println!("  {} - Exit", "'exit'".yellow());
    println!();

    let mut rodeo = Rodeo::new();
    let stdin = io::stdin();

    loop {
        print!("{} ", ">>".cyan());
        io::Write::flush(&mut io::stdout()).unwrap();

        let mut input = String::new();
        if stdin.read_line(&mut input).is_err() {
            eprintln!("{}", "[ERROR]: Failed to read input".red());
            continue;
        }

        let input = input.trim();

        match input {
            "" => continue,
            "exit" => {
                println!("{}", "Goodbye!".green());
                break;
            }
            cmd if cmd.starts_with("load ") => {
                let file_path = &cmd[5..];
                if let Err(err) = execute_file(file_path) {
                    eprintln!("{}", format!("[ERROR]: {}", err).red());
                }
            }
            _ => {
                execute(&mut rodeo, input, "<stdin>");
            }
        }
    }
}

pub fn execute_file(file_path: &str) -> io::Result<()> {
    let input = fs::read_to_string(file_path)?;
    let mut rodeo = Rodeo::new();
    execute(&mut rodeo, input.trim(), file_path);
    Ok(())
}

fn execute(rodeo: &mut Rodeo, input: &str, filename: &str) {
    let source = SourceCode::new(input.to_string(), filename.to_string());

    // Building phase
    if let Err(errors) = build(rodeo, &source) {
        for error in errors {
            eprintln!("{}", error);
        }
    }
}

fn build(rodeo: &mut Rodeo, source: &SourceCode) -> Result<(), Vec<CompilerError>> {
    // Lexing phase
    let lexer = Lexer::new(rodeo, source);
    let lexer_output = lexer.scan();

    if !lexer_output.errors.is_empty() {
        return Err(lexer_output.errors);
    }
    println!("{}", "[INFO]: Lexing complete.".green());

    // Parsing phase
    let parser = Parser::new(lexer_output, source);
    let parser_output = parser.parse();

    if !parser_output.errors.is_empty() {
        return Err(parser_output.errors);
    }
    println!("{}", "[INFO]: Parsing complete.".green());

    // TODO This is debug output
    // Print AST
    for node in parser_output.nodes {
        println!("{}", node.print(rodeo));
    }

    Ok(())
}
