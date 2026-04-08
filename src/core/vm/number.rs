#![allow(clippy::should_implement_trait)]
use crate::core::vm::error::VmError;
use std::cmp::Ordering;
use std::fmt::Display;

/// Macro for binary operations
macro_rules! binary_op {
    ($name:ident, $op:expr) => {
        pub fn $name(self, other: Self) -> Self {
            match (self, other) {
                (Number::Int(a), Number::Int(b)) => Number::Int($op(a, b)),
                (Number::Float(a), Number::Float(b)) => Number::Float($op(a, b)),
                (Number::Int(a), Number::Float(b)) => Number::Float($op(a as f32, b)),
                (Number::Float(a), Number::Int(b)) => Number::Float($op(a, b as f32)),
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    Int(i32),
    Float(f32),
}

impl Eq for Number {}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Number {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            // Both integers
            (Number::Int(a), Number::Int(b)) => a.cmp(b),

            // Both floats
            (Number::Float(a), Number::Float(b)) => {
                // Handle NaN: treat NaN as greater than all other values
                match (a.is_nan(), b.is_nan()) {
                    (true, true) => Ordering::Equal,
                    (true, false) => Ordering::Greater,
                    (false, true) => Ordering::Less,
                    (false, false) => {
                        if a < b {
                            Ordering::Less
                        } else if a > b {
                            Ordering::Greater
                        } else {
                            Ordering::Equal
                        }
                    }
                }
            }

            // Int vs Float: convert int to float and compare
            (Number::Int(a), Number::Float(b)) => {
                let a_float = *a as f32;
                if a_float < *b || b.is_nan() {
                    Ordering::Less
                } else if a_float > *b {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            }

            // Float vs Int: convert int to float and compare
            (Number::Float(a), Number::Int(b)) => {
                let b_float = *b as f32;
                if a.is_nan() {
                    Ordering::Greater
                } else if a < &b_float {
                    Ordering::Less
                } else if a > &b_float {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            }
        }
    }
}

impl Number {
    binary_op!(add, |a, b| a + b);
    binary_op!(subtract, |a, b| a - b);
    binary_op!(multiply, |a, b| a * b);
    binary_op!(divide, |a, b| a / b);
    binary_op!(modulo, |a, b| a % b);
    pub fn pow(self, other: Self) -> Self {
        match (self, other) {
            // Int ^ Int = Int (if exponent is non-negative)
            (Number::Int(base), Number::Int(exp)) => {
                if exp < 0 {
                    Number::Float((base as f32).powf(exp as f32))
                } else {
                    Number::Int(base.pow(exp as u32))
                }
            }
            // Everything else - float
            (Number::Float(base), Number::Float(exp)) => Number::Float(base.powf(exp)),
            (Number::Int(base), Number::Float(exp)) => Number::Float((base as f32).powf(exp)),
            (Number::Float(base), Number::Int(exp)) => Number::Float(base.powf(exp as f32)),
        }
    }
    pub fn negate(self) -> Self {
        match self {
            Number::Int(a) => Number::Int(-a),
            Number::Float(a) => Number::Float(-a),
        }
    }

    pub fn to_f32(&self) -> f32 {
        match self {
            Number::Int(i) => *i as f32,
            Number::Float(f) => *f,
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            Number::Int(i) => *i,
            Number::Float(f) => *f as i32,
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Int(num) => write!(f, "{}", num),
            Number::Float(num) => write!(f, "{}", num),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Imm(Number),
    Ref(usize),
    StackRef {
        frame_index: usize,
        local_index: usize,
    },
}

impl Value {
    pub fn as_num(&self) -> Result<Number, VmError> {
        match self {
            Value::Imm(i) => Ok(*i),
            Value::Ref(_) => Err(VmError::TypeMismatch(
                "reference".to_string(),
                "integer".to_string(),
            )),
            Value::StackRef { .. } => Err(VmError::TypeMismatch(
                "stack reference".to_string(),
                "integer".to_string(),
            )),
        }
    }

    pub fn as_ref(&self) -> Result<usize, VmError> {
        match self {
            Value::Ref(addr) => Ok(*addr),
            Value::StackRef { .. } => Err(VmError::TypeMismatch(
                "stack reference".to_string(),
                "reference".to_string(),
            )),
            Value::Imm(_) => Err(VmError::TypeMismatch(
                "immediate".to_string(),
                "reference".to_string(),
            )),
        }
    }

    pub fn is_ref(&self) -> bool {
        matches!(self, Value::Ref(_))
    }

    pub fn is_imm(&self) -> bool {
        matches!(self, Value::Imm(_))
    }
}
