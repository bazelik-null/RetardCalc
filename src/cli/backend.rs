// Copyright (c) 2026 bazelik-null

use crate::morsel_interpreter::Interpreter;
use std::time::Instant;
use std::{fs, io};

pub fn cli_execute(input: &str, is_debug: bool) -> Result<(), String> {
    let start = Instant::now();

    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(input);

    let elapsed = start.elapsed();

    if is_debug {
        println!(
            "[DEBUG]: Execution time: {:.4}ms ({:?})",
            elapsed.as_secs_f64() * 1000.0,
            elapsed
        );
    }

    result
}
/// Reads file and evaluates it.
pub fn eval_file(file_path: &str, is_debug: bool) -> io::Result<()> {
    let input = fs::read_to_string(file_path)?;
    cli_execute(input.trim(), is_debug).map_err(io::Error::other)?;
    Ok(())
}
