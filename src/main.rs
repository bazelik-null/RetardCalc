// Copyright (c) 2026 bazelik-null

pub mod cli;
pub mod interpreter;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // If filepath passed just eval the file.
    if args.len() > 1 {
        if let Err(err) = cli::backend::eval_file(&args[1], false) {
            eprintln!("[ERROR]: {}", err);
            std::process::exit(1);
        }
    }
    // Else enter CLI
    else {
        cli::cli_main::cli_init();
    }
}
