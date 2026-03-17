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
    Sqrt,     // √ x
    Log,      // x log(y) [where x is base, y is argument]
    Ln,       // ln(x)
    // Trigonometry
    Cos,  // cos(x)
    Sin,  // sin(x)
    Tan,  // tan(x)
    Acos, // arccos(x)
    Asin, // arcsin(x)
    Atan, // arctan(x)
    // Misc
    Negate, // -x
    Modulo, // x % y
    Abs,    // abs(x)
    Round,  // round(x)
    // Parenthesis
    LParen, // (
    RParen, // )

    #[default]
    Unknown,
}

impl OperatorType {
    /// Returns precedence for binary operators
    pub fn precedence(&self) -> Option<Precedence> {
        Some(match self {
            Self::Add | Self::Subtract => Precedence::Additive,
            Self::Multiply | Self::Divide | Self::Modulo | Self::Log => Precedence::Multiplicative,
            Self::Exponent => Precedence::Exponent,
            _ => return None,
        })
    }

    pub fn is_right_associative(&self) -> bool {
        matches!(self, Self::Exponent)
    }

    pub fn is_function(&self) -> bool {
        matches!(
            self,
            Self::Sqrt
                | Self::Ln
                | Self::Cos
                | Self::Sin
                | Self::Tan
                | Self::Acos
                | Self::Asin
                | Self::Atan
                | Self::Abs
                | Self::Round
        )
    }

    pub fn is_unary(&self) -> bool {
        matches!(self, Self::Negate)
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
            OperatorType::Sqrt => write!(f, "√"),
            OperatorType::Log => write!(f, "log"),
            OperatorType::Ln => write!(f, "ln"),
            // Trigonometry
            OperatorType::Cos => write!(f, "cos"),
            OperatorType::Sin => write!(f, "sin"),
            OperatorType::Tan => write!(f, "tan"),
            OperatorType::Acos => write!(f, "arccos"),
            OperatorType::Asin => write!(f, "arcsin"),
            OperatorType::Atan => write!(f, "arctan"),
            // Misc
            OperatorType::Negate => write!(f, "-"),
            OperatorType::Modulo => write!(f, "%"),
            OperatorType::Abs => write!(f, "abs"),
            OperatorType::Round => write!(f, "round"),
            // Brackets
            OperatorType::LParen => write!(f, "("),
            OperatorType::RParen => write!(f, ")"),

            OperatorType::Unknown => write!(f, "?"),
        }
    }
}
