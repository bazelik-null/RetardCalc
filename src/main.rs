pub mod core;

use crate::core::compiler::codegen::CodeGenerator;
use crate::core::compiler::error_handler::CompilerError;
use crate::core::compiler::parser::Parser;
use crate::core::compiler::preprocessor::lexer::Lexer;
use crate::core::compiler::source::SourceCode;
use crate::core::shared::executable::Executable;
use crate::core::tools::disassembler::Disassembler;
use crate::core::vm::VirtualMachine;
use colored::Colorize;
use lasso::Rodeo;
use std::time::Instant;
use std::{env, fs, io, path::Path, process};

const EXE_EXTENSION: &str = "msle";
const SOURCE_EXTENSION: &str = "msl";
const HEAP_SIZE: usize = 8000000; // 8MB

#[derive(Debug)]
enum Command {
    Build(String),
    Disassemble(String),
    Run(String, bool),
    BuildAndRun(String, bool),
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Err(err) = execute_command(&args) {
        eprintln!("{}", format!("[ERROR]: {}", err).red());
        process::exit(1);
    }
}

fn execute_command(args: &[String]) -> Result<(), String> {
    let command = parse_command(args).inspect_err(|_| {
        print_usage();
    })?;

    match command {
        Command::Build(path) => handle_build(&path).map_err(|e| e.to_string()),
        Command::Disassemble(path) => handle_disassemble(&path).map_err(|e| e.to_string()),
        Command::Run(path, debug) => handle_run(&path, debug).map_err(|e| e.to_string()),
        Command::BuildAndRun(mut path, debug) => {
            handle_build(&path).map_err(|e| e.to_string())?;
            path += "e"; // Add e to extension
            println!(); // Print newline
            handle_run(&path, debug).map_err(|e| e.to_string())
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
            let path = args
                .get(2)
                .ok_or_else(|| format!("'{}' requires a path argument", command))?;
            Ok(Command::Build(path.clone()))
        }
        "disassemble" => {
            let path = args
                .get(2)
                .ok_or_else(|| format!("'{}' requires a path argument", command))?;
            Ok(Command::Disassemble(path.clone()))
        }
        "run" => {
            if args.len() < 3 {
                return Err(format!("'{}' requires a path argument", command));
            }

            let mut build = false;
            let mut debug = false;
            let mut path_opt: Option<String> = None;

            for token in args.iter().skip(2) {
                match token.as_str() {
                    "--build" => build = true,
                    "--debug" => debug = true,
                    other => {
                        if path_opt.is_none() {
                            path_opt = Some(other.to_string());
                        } else {
                            return Err(format!("Unexpected argument '{}'", other));
                        }
                    }
                }
            }

            let path = path_opt.ok_or_else(|| format!("'{}' requires a path argument", command))?;

            if build {
                Ok(Command::BuildAndRun(path, debug))
            } else {
                Ok(Command::Run(path, debug))
            }
        }
        _ => Err(format!("unknown command: '{}'", command)),
    }
}

fn print_usage() {
    println!("{}", "USAGE:".cyan().bold());
    println!("    morsel <COMMAND> <ARG> <PATH>");
    println!();
    println!("{}", "COMMANDS:".cyan().bold());
    println!(
        "    build <PATH>                     Build a .{} file to .{} executable",
        SOURCE_EXTENSION, EXE_EXTENSION
    );
    println!(
        "    disassemble <PATH>               Disassemble a .{} executable",
        EXE_EXTENSION
    );
    println!(
        "    run <--build> <--debug> <PATH>   Execute a .{} file (and build if specified)",
        EXE_EXTENSION
    );
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

fn validate_extension(file_path: &str, expected: &str) -> io::Result<()> {
    if !file_path.ends_with(&format!(".{}", expected)) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Input file must have .{} extension", expected),
        ));
    }
    Ok(())
}

fn read_and_deserialize_executable(file_path: &str) -> io::Result<Executable> {
    validate_extension(file_path, EXE_EXTENSION)?;
    let bytes = fs::read(file_path)?;
    Executable::deserialize(&bytes).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

fn time_phase<F, T>(phase_name: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    println!(
        "{}",
        format!(
            "[INFO]: {} complete. ({:.2}ms)",
            phase_name,
            duration.as_secs_f64() * 1000.0
        )
        .green()
    );
    result
}

fn handle_build(file_path: &str) -> io::Result<()> {
    validate_extension(file_path, SOURCE_EXTENSION)?;

    let input = fs::read_to_string(file_path)?;
    let mut rodeo = Rodeo::new();
    let source = SourceCode::new(input.trim().to_string(), file_path.to_string());

    match build(&mut rodeo, &source) {
        Ok(executable) => {
            let output_path = get_output_path(file_path, EXE_EXTENSION);
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
            Err(io::Error::other("Compilation failed"))
        }
    }
}

fn handle_disassemble(file_path: &str) -> io::Result<()> {
    let exe = read_and_deserialize_executable(file_path)?;
    let disassembly = Disassembler::disassemble(&exe);
    println!("{}", disassembly);
    Ok(())
}

fn handle_run(file_path: &str, debug: bool) -> io::Result<()> {
    let exe = read_and_deserialize_executable(file_path)?;
    run(exe, debug).map_err(io::Error::other)
}

fn run(executable: Executable, debug: bool) -> Result<(), String> {
    // Load VM and executable
    let mut virtual_machine = VirtualMachine::new(HEAP_SIZE);
    virtual_machine
        .load_executable(&executable)
        .map_err(|err| err.to_string())?;

    // Execute program
    match debug {
        true => {
            println!("{}", "[INFO]: Running in debug mode.".to_string().cyan());
            virtual_machine.run_debug().map_err(|err| err.to_string())?;
        }
        false => {
            virtual_machine.run().map_err(|err| err.to_string())?;
        }
    }

    Ok(())
}

fn build(rodeo: &mut Rodeo, source: &SourceCode) -> Result<Executable, Vec<CompilerError>> {
    let build_start = Instant::now();

    // Lexing phase
    let lexer_output = time_phase("Lexing", || {
        let lexer = Lexer::new(rodeo, source);
        lexer.scan()
    });

    if !lexer_output.errors.is_empty() {
        return Err(lexer_output.errors);
    }

    // Parsing phase
    let parser_output = time_phase("Parsing", || {
        let parser = Parser::new(lexer_output, source, rodeo);
        parser.parse()
    });

    if !parser_output.errors.is_empty() {
        return Err(parser_output.errors);
    }

    // Code generation phase
    let exe = time_phase("Code generation", || {
        let compiler = CodeGenerator::new(rodeo);
        compiler.compile(&parser_output.nodes)
    })
    .map_err(|e| {
        vec![string_to_compiler_error(
            e,
            "Code generator",
            source.filename.clone(),
        )]
    })?;

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

fn string_to_compiler_error(err: String, from: &str, filename: String) -> CompilerError {
    CompilerError::new(err, from.to_string(), 0, 0, 0, None, Some(filename))
}
