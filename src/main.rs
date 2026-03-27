pub mod core;

use crate::core::compiler::codegen::CodeGenerator;
use crate::core::compiler::error_handler::CompilerError;
use crate::core::compiler::parser::Parser;
use crate::core::compiler::preprocessor::lexer::Lexer;
use crate::core::compiler::source::SourceCode;
use crate::core::shared::executable::Executable;
use crate::core::tools::disassembler::Disassembler;
use colored::Colorize;
use lasso::Rodeo;
use std::time::Instant;
use std::{env, fs, io, path::Path, process};

#[derive(Debug)]
enum Command {
    Build(String),
    Disassemble(String),
    Run(String),
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match parse_command(&args) {
        Ok(command) => match command {
            Command::Build(path) => {
                if let Err(err) = handle_build(&path) {
                    eprintln!("{}", format!("[ERROR]: {}", err).red());
                    process::exit(1);
                }
            }
            Command::Disassemble(path) => {
                if let Err(err) = handle_disassemble(&path) {
                    eprintln!("{}", format!("[ERROR]: {}", err).red());
                    process::exit(1);
                }
            }
            Command::Run(_path) => {
                todo!()
            }
        },
        Err(err) => {
            eprintln!("{}", err.red());
            print_usage();
            process::exit(1);
        }
    }
}

fn parse_command(args: &[String]) -> Result<Command, String> {
    if args.len() < 2 {
        return Err("No command specified".to_string());
    }

    let command = &args[1];

    match command.as_str() {
        "build" => {
            if args.len() < 3 {
                return Err("'build' requires a path argument".to_string());
            }
            Ok(Command::Build(args[2].clone()))
        }
        "disassemble" => {
            if args.len() < 3 {
                return Err("'disassemble' requires a path argument".to_string());
            }
            Ok(Command::Disassemble(args[2].clone()))
        }
        "run" => {
            if args.len() < 3 {
                return Err("'run' requires a path argument".to_string());
            }
            Ok(Command::Run(args[2].clone()))
        }
        _ => Err(format!("unknown command: '{}'", command)),
    }
}

fn print_usage() {
    println!("{}", "USAGE:".cyan().bold());
    println!("    morsel <COMMAND> <PATH>");
    println!();
    println!("{}", "COMMANDS:".cyan().bold());
    println!("    build <PATH>       Build a .msl file to .msle executable");
    println!("    disassemble <PATH> Disassemble a .msle executable");
    println!("    run <PATH>         Execute a .msle file");
}

fn get_output_path(input_path: &str, extension: &str) -> String {
    let path = Path::new(input_path);
    let stem = path.file_stem().unwrap().to_string_lossy();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    parent
        .join(format!("{}.{}", stem, extension))
        .to_string_lossy()
        .to_string()
}

fn handle_build(file_path: &str) -> io::Result<()> {
    // Validate input file extension
    if !file_path.ends_with(".msl") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Input file must have .msl extension",
        ));
    }

    let input = fs::read_to_string(file_path)?;
    let mut rodeo = Rodeo::new();
    let source = SourceCode::new(input.trim().to_string(), file_path.to_string());

    match build(&mut rodeo, &source) {
        Ok(executable) => {
            // Save executable to .msle file
            let output_path = get_output_path(file_path, "msle");
            let serialized = executable.serialize();
            fs::write(&output_path, serialized)?;

            println!(
                "{}",
                format!("[INFO]: Executable saved to {}", output_path).green()
            );
            Ok(())
        }
        Err(errors) => {
            for error in errors {
                eprintln!("{}", error);
            }
            process::exit(1);
        }
    }
}

fn handle_disassemble(file_path: &str) -> io::Result<()> {
    // Support .msle files
    if file_path.ends_with(".msle") {
        let bytes = fs::read(file_path)?;
        // Try to deserialize as executable
        match Executable::deserialize(&bytes) {
            Ok(exe) => {
                let disassembly = Disassembler::disassemble(&exe);
                println!("{}", disassembly);
                Ok(())
            }
            Err(err) => Err(io::Error::new(io::ErrorKind::InvalidData, err)),
        }
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Input file must have .msle extension",
        ))
    }
}

fn build(rodeo: &mut Rodeo, source: &SourceCode) -> Result<Executable, Vec<CompilerError>> {
    let build_start = Instant::now();

    // Lexing phase
    let lexing_start = Instant::now();
    let lexer = Lexer::new(rodeo, source);
    let lexer_output = lexer.scan();

    if !lexer_output.errors.is_empty() {
        return Err(lexer_output.errors);
    }
    let lexing_duration = lexing_start.elapsed();
    println!(
        "{}",
        format!(
            "[INFO]: Lexing complete. ({:.2}ms)",
            lexing_duration.as_secs_f64() * 1000.0
        )
        .green()
    );

    // Parsing phase
    let parsing_start = Instant::now();
    let parser = Parser::new(lexer_output, source, rodeo);
    let parser_output = parser.parse();

    if !parser_output.errors.is_empty() {
        return Err(parser_output.errors);
    }
    let parsing_duration = parsing_start.elapsed();
    println!(
        "{}",
        format!(
            "[INFO]: Parsing complete. ({:.2}ms)",
            parsing_duration.as_secs_f64() * 1000.0
        )
        .green()
    );

    // Code generation phase
    let codegen_start = Instant::now();
    let compiler = CodeGenerator::new(rodeo);
    match compiler.compile(&parser_output.nodes) {
        Ok(exe) => {
            let codegen_duration = codegen_start.elapsed();
            println!(
                "{}",
                format!(
                    "[INFO]: Compilation complete. ({:.2}ms)",
                    codegen_duration.as_secs_f64() * 1000.0
                )
                .green()
            );

            let total_duration = build_start.elapsed();
            println!(
                "{}",
                format!(
                    "[INFO]: Total build time: {:.2}ms",
                    total_duration.as_secs_f64() * 1000.0
                )
                .green()
            );

            Ok(exe)
        }
        Err(error) => {
            eprintln!("{}", error);
            Err(vec![])
        }
    }
}
