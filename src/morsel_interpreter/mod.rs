// Copyright (c) 2026 bazelik-null

pub mod environment;
pub mod lexer;
pub mod parser;
pub mod runtime;

//
// Interpreter
//

use crate::morsel_interpreter::environment::symbol_table::SymbolTable;
use crate::morsel_interpreter::lexer::tokenizer;
use crate::morsel_interpreter::parser::builder::AstBuilder;
use crate::morsel_interpreter::runtime::executor::Executor;

pub struct Interpreter {
    symbol_table: SymbolTable,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            symbol_table: SymbolTable::new(),
        }
    }

    /// Execute expression
    pub fn execute(&mut self, input: &str) -> Result<(), String> {
        // Tokenize
        let tokens = tokenizer::tokenize(input)?;

        // Parse into AST
        let parser = AstBuilder::new(&mut self.symbol_table, tokens);
        parser.build()?;

        // Execute program
        let mut evaluator = Executor::new(&mut self.symbol_table);
        evaluator.execute()?;
        Ok(())
    }
}
