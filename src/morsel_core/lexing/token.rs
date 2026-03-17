// Copyright (c) 2026 bazelik-null

use crate::morsel_core::lexing::operators::OperatorType;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),            // Plain numbers
    Operator(OperatorType), // Operators like '+', '-'
    Function(String),       // Functions
    Keyword(String),        // For 'let', 'if', 'else', etc.
    Identifier(String),     // For variable names
}

impl Token {
    pub fn is_number(&self) -> bool {
        matches!(self, Token::Number(_))
    }
    pub fn is_operator(&self) -> bool {
        matches!(self, Token::Operator(_))
    }
    pub fn is_function(&self) -> bool {
        matches!(self, Token::Function(_))
    }
    pub fn is_keyword(&self) -> bool {
        matches!(self, Token::Keyword(_))
    }
    pub fn is_identifier(&self) -> bool {
        matches!(self, Token::Identifier(_))
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Token::Number(value) => Some(*value),
            _ => None,
        }
    }
    pub fn as_operator(&self) -> Option<&OperatorType> {
        match self {
            Token::Operator(op) => Some(op),
            _ => None,
        }
    }
    pub fn as_function(&self) -> Option<&String> {
        match self {
            Token::Function(func) => Some(func),
            _ => None,
        }
    }
    pub fn as_keyword(&self) -> Option<&String> {
        match self {
            Token::Keyword(keyword) => Some(keyword),
            _ => None,
        }
    }
    pub fn as_identifier(&self) -> Option<&String> {
        match self {
            Token::Identifier(name) => Some(name),
            _ => None,
        }
    }
}
