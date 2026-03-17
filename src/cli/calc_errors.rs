// Copyright (c) 2026 bazelik-null

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CalcError {
    Tokenize(String),
    Parse(String),
    Evaluate(String),
    IoError(String),
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CalcError::Tokenize(msg) => write!(f, "Tokenization error: {}", msg),
            CalcError::Parse(msg) => write!(f, "Parse error: {}", msg),
            CalcError::Evaluate(msg) => write!(f, "Evaluation error: {}", msg),
            CalcError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl Error for CalcError {}
