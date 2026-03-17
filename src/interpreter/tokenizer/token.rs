// Copyright (c) 2026 bazelik-null

use crate::interpreter::operators::OperatorType;

#[derive(Debug, Copy, Clone)]
pub enum Token {
    Operator(OperatorType),
    Number(f64),
}

impl Token {
    pub fn is_number(&self) -> bool {
        matches!(self, Token::Number(_))
    }

    pub fn is_operator(&self) -> bool {
        matches!(self, Token::Operator(_))
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
}
