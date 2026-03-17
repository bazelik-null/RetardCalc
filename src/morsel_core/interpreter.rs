// Copyright (c) 2026 bazelik-null

use crate::morsel_core::evaluating::evaluator::Evaluator;
use crate::morsel_core::evaluating::functions::FunctionTable;
use crate::morsel_core::lexing::lexer::Lexer;
use crate::morsel_core::lexing::token::Token;
use crate::morsel_core::parsing::node::Node;
use crate::morsel_core::parsing::parser::Parser;

use std::sync::Arc;

pub struct Interpreter {
    func_table: Arc<FunctionTable>,
    debug: bool,
}

impl Interpreter {
    pub fn new(debug: bool) -> Self {
        Interpreter {
            func_table: Arc::new(FunctionTable::new()),
            debug,
        }
    }

    /// Execute expression
    pub fn execute(&self, input: &str) -> Result<(), String> {
        if self.debug {
            println!("[DEBUG]: Raw input: {}", input);
        }

        // Tokenize
        let tokens = self.tokenize(input)?;
        if self.debug {
            println!("[DEBUG]: Tokens: {:?}", tokens);
        }

        // Parse into AST
        let ast = self.parse(tokens)?;
        if self.debug {
            println!("[DEBUG]: Abstract Syntax Tree:\n {}", ast);
        }

        // Evaluate AST
        self.evaluate(&ast)
    }

    /// Execute expression
    pub fn execute_with_result(&self, input: &str) -> Result<f64, String> {
        if self.debug {
            println!("[DEBUG]: Raw input: {}", input);
        }

        // Tokenize
        let tokens = self.tokenize(input)?;
        if self.debug {
            println!("[DEBUG]: Tokens: {:?}", tokens);
        }

        // Parse into AST
        let ast = self.parse(tokens)?;
        if self.debug {
            println!("[DEBUG]: Abstract Syntax Tree:\n {}", ast);
        }

        // Evaluate AST
        self.evaluate_with_result(&ast)
    }

    /// Tokenize input string into tokens
    fn tokenize(&self, input: &str) -> Result<Vec<Token>, String> {
        let lexer = Lexer::new(self.func_table.clone());
        lexer.tokenize(input)
    }

    /// Parse tokens into an Abstract Syntax Tree
    fn parse(&self, tokens: Vec<Token>) -> Result<Node, String> {
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    /// Evaluate an AST node
    fn evaluate(&self, ast: &Node) -> Result<(), String> {
        let mut evaluator = Evaluator::new(self.func_table.clone());
        evaluator.eval(ast)?;
        Ok(())
    }

    /// Evaluate an AST node and return result
    fn evaluate_with_result(&self, ast: &Node) -> Result<f64, String> {
        let mut evaluator = Evaluator::new(self.func_table.clone());
        evaluator.eval(ast)
    }

    /// Get the function table
    pub fn function_table(&self) -> &Arc<FunctionTable> {
        &self.func_table
    }

    /// Enable or disable debug output
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }
}
