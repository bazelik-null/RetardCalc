// Copyright (c) 2026 bazelik-null

use std::fmt;

/// Lower numbers = lower precedence
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Additive = 1,
    Multiplicative = 2,
    Exponent = 3,
}

impl Precedence {
    pub fn next_higher(self) -> Self {
        match self {
            Precedence::Additive => Precedence::Multiplicative,
            Precedence::Multiplicative => Precedence::Exponent,
            Precedence::Exponent => Precedence::Exponent,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum OperatorType {
    // Arithmetic
    Add,      // x + y
    Subtract, // x - y
    Multiply, // x * y
    Divide,   // x / y
    // Exponents
    Exponent, // x ^ y
    // Misc
    Negate, // -x
    Modulo, // x % y
    // Syntax
    LParen,    // (
    RParen,    // )
    Comma,     // ,
    Assign,    // =
    Semicolon, // ;

    #[default]
    Unknown,
}

impl OperatorType {
    /// Returns precedence for binary operators
    pub fn precedence(&self) -> Option<Precedence> {
        Some(match self {
            Self::Add | Self::Subtract => Precedence::Additive,
            Self::Multiply | Self::Divide | Self::Modulo => Precedence::Multiplicative,
            Self::Exponent => Precedence::Exponent,
            _ => return None,
        })
    }

    pub fn is_right_associative(&self) -> bool {
        matches!(self, Self::Exponent)
    }

    pub fn is_unary(&self) -> bool {
        matches!(self, Self::Negate)
    }

    pub fn is_postfix(&self) -> bool {
        false
    }
}

impl fmt::Display for OperatorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Arithmetic
            OperatorType::Add => write!(f, "+"),
            OperatorType::Subtract => write!(f, "-"),
            OperatorType::Multiply => write!(f, "*"),
            OperatorType::Divide => write!(f, "/"),
            // Exponents
            OperatorType::Exponent => write!(f, "^"),
            // Misc
            OperatorType::Negate => write!(f, "-"),
            OperatorType::Modulo => write!(f, "%"),
            // Syntax
            OperatorType::LParen => write!(f, "("),
            OperatorType::RParen => write!(f, ")"),
            OperatorType::Comma => write!(f, ","),
            OperatorType::Assign => write!(f, "="),
            OperatorType::Semicolon => write!(f, ";"),

            OperatorType::Unknown => write!(f, "?"),
        }
    }
}
