// Copyright (c) 2026 bazelik-null

use crate::interpreter::ast::parser;
use crate::interpreter::evaluator;
use crate::interpreter::tokenizer::lexer;
use std::{fs, io};

use crate::cli::calc_errors::CalcError;

/// Takes a raw input string and:
/// 1. Parses string into Tokens array.
/// 2. Builds Abstract Syntax Tree (AST) from Tokens.
/// 3. Evaluates AST Nodes and returns result.
pub fn calculate(input: &str, is_debug: bool) -> Result<f64, CalcError> {
    if is_debug {
        println!("[DEBUG]: Raw input: {}", input);
    }

    // Tokenize
    let tokens = lexer::tokenize(input).map_err(|e| CalcError::Tokenize(e.to_string()))?;
    if is_debug {
        println!("[DEBUG]: Tokens: {:?}", tokens);
    }

    // Parse into AST
    let mut parser = parser::Parser::new(tokens);
    let ast = parser
        .parse()
        .map_err(|e| CalcError::Parse(e.to_string()))?;
    if is_debug {
        println!("[DEBUG]: Raw AST: {:?}", ast);
        println!("[DEBUG]: Pretty AST: {}", ast);
    }

    // Evaluate AST
    let result = evaluator::eval(&ast).map_err(|e| CalcError::Evaluate(e.to_string()))?;

    Ok(result)
}

/// Reads file and evaluates it.
pub fn eval_file(file_path: &str, is_debug: bool) -> io::Result<()> {
    let input = fs::read_to_string(file_path)?;
    let result = calculate(input.trim(), is_debug).map_err(io::Error::other)?;
    println!("{}", result);
    Ok(())
}
