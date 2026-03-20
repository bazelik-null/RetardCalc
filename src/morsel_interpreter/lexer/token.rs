// Copyright (c) 2026 bazelik-null

use crate::morsel_interpreter::lexer::syntax_operator::SyntaxOperator;

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Integer(i64),
    Float(f64),
    String(Box<str>),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Literal(LiteralValue),       // Plain numbers
    SyntaxToken(SyntaxOperator), // Operators and tokens like '+', '-', '()' '{}'
    Keyword(String),             // For 'let', 'if', 'else', etc.
    Identifier(String),          // For references and function calls
    Type(String),                // For variable types
}

impl Token {
    pub fn as_operator(&self) -> Option<&SyntaxOperator> {
        match self {
            Token::SyntaxToken(op) => Some(op),
            _ => None,
        }
    }
}
