// Copyright (c) 2026 bazelik-null

pub mod environment;
pub mod lexer;
pub mod parser;
pub mod runtime;

//
// Interpreter
//

use crate::morsel_interpreter::environment::symbol_table::SymbolTable;
use crate::morsel_interpreter::lexer::token::Token;
use crate::morsel_interpreter::lexer::tokenizer;
use crate::morsel_interpreter::parser::builder::AstBuilder;
use crate::morsel_interpreter::runtime::executor::Executor;

pub struct Interpreter {
    symbol_table: SymbolTable,
    debug: bool,
}

impl Interpreter {
    pub fn new(debug: bool) -> Self {
        Interpreter {
            symbol_table: SymbolTable::new(),
            debug,
        }
    }

    /// Execute expression
    pub fn execute(&mut self, input: &str) -> Result<(), String> {
        if self.debug {
            println!();
            println!("[DEBUG]: Raw input: \n{}", input);
            println!();
        }

        // Tokenize
        let tokens = self.tokenize(input)?;

        if self.debug {
            println!("[DEBUG]: Tokens: {:?}", tokens);
        }

        // Parse into AST
        self.symbol_table = self.parse(tokens)?;

        // Evaluate AST
        self.evaluate()
    }

    /// Tokenize input string into tokens
    fn tokenize(&self, input: &str) -> Result<Vec<Token>, String> {
        tokenizer::tokenize(input)
    }

    /// Parse tokens into an Abstract Syntax Tree
    fn parse(&self, tokens: Vec<Token>) -> Result<SymbolTable, String> {
        let parser = AstBuilder::new(self.symbol_table.clone(), tokens);
        parser.build()
    }

    /// Evaluate an AST node
    fn evaluate(&mut self) -> Result<(), String> {
        let mut evaluator = Executor::new(self.symbol_table.clone());
        evaluator.execute()?;
        Ok(())
    }

    /// Enable or disable debug output
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }
}
