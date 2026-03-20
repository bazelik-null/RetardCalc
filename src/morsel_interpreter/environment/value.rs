// Copyright (c) 2026 bazelik-null

use crate::morsel_interpreter::environment::types::Type;

/// Represents any value in the language
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Float(f64),
    Integer(i64),
    String(String),
    Boolean(bool),
    Null,
}

impl Value {
    /// Get the type of this value
    pub fn type_of(&self) -> Type {
        Type::of(self)
    }

    /// Convert to f64 (works with any type)
    pub fn to_float(&self) -> Result<f64, String> {
        match self {
            Value::Float(f) => Ok(*f),
            Value::Integer(i) => Ok(*i as f64),
            Value::String(s) => s
                .parse::<f64>()
                .map_err(|_| format!("Cannot convert string '{}' to float", s)),
            Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            Value::Null => Err("Cannot convert null to float".to_string()),
        }
    }

    /// Convert to integer (works with any type)
    pub fn to_integer(&self) -> Result<i64, String> {
        match self {
            Value::Integer(i) => Ok(*i),
            Value::Float(f) => Ok(*f as i64),
            Value::String(s) => s
                .parse::<i64>()
                .map_err(|_| format!("Cannot convert string '{}' to integer", s)),
            Value::Boolean(b) => Ok(if *b { 1 } else { 0 }),
            Value::Null => Err("Cannot convert null to integer".to_string()),
        }
    }

    /// Convert to boolean (works with any type)
    pub fn to_bool(&self) -> Result<bool, String> {
        Ok(self.is_truthy())
    }

    /// Convert to string for display purposes
    /// Used by the print/println functions. Strings are returned without quotes.
    pub fn display(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Float(f) => {
                if f.fract() == 0.0 && f.is_finite() {
                    format!("{:.1}", f)
                } else {
                    f.to_string()
                }
            }
            Value::Integer(i) => i.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".to_string(),
        }
    }

    /// Check if this value is truthy (for conditionals)
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Integer(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
        }
    }

    /// Check if this value is falsy
    pub fn is_falsy(&self) -> bool {
        !self.is_truthy() // Yeah
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}
