use crate::core::compiler::preprocessor::token::{LiteralValue, OperatorValue};
use crate::core::shared::types::Type;

pub fn infer_literal_type(lit: &LiteralValue) -> Type {
    match lit {
        LiteralValue::Integer(_) => Type::Integer,
        LiteralValue::Float(_) => Type::Float,
        LiteralValue::Boolean(_) => Type::Boolean,
        LiteralValue::String(_) => Type::String,
    }
}

pub fn infer_binary_type(
    lhs: &Type,
    op: &OperatorValue,
    rhs: &Type,

    errors: &mut Vec<String>,
) -> Result<Type, ()> {
    match op {
        OperatorValue::Plus
        | OperatorValue::Minus
        | OperatorValue::Multiply
        | OperatorValue::Divide
        | OperatorValue::Modulo
        | OperatorValue::Power => arithmetic_type(lhs, rhs, op, errors),

        OperatorValue::Less
        | OperatorValue::LessEqual
        | OperatorValue::Greater
        | OperatorValue::GreaterEqual
        | OperatorValue::Equal
        | OperatorValue::NotEqual => comparison_type(lhs, rhs, errors),

        OperatorValue::And | OperatorValue::Or => logical_type(lhs, rhs, errors),

        OperatorValue::Xor => bitwise_type(lhs, rhs, op, errors),

        OperatorValue::ShiftLeft | OperatorValue::ShiftRight => shift_type(lhs, rhs, errors),

        _ => {
            errors.push(format!("Unsupported binary operator: {}", op));
            Err(())
        }
    }
}

pub fn infer_unary_type(
    op: &OperatorValue,
    rhs: &Type,
    errors: &mut Vec<String>,
) -> Result<Type, ()> {
    match op {
        OperatorValue::Minus => match rhs {
            Type::Integer | Type::Float => Ok(rhs.clone()),
            _ => {
                errors.push(format!("Cannot negate type {}", rhs));
                Err(())
            }
        },
        OperatorValue::Not => {
            if rhs == &Type::Boolean {
                Ok(Type::Boolean)
            } else {
                errors.push(format!("Logical NOT requires boolean, got {}", rhs));
                Err(())
            }
        }
        _ => {
            errors.push(format!("Unsupported unary operator: {}", op));
            Err(())
        }
    }
}

fn bitwise_type(
    lhs: &Type,
    rhs: &Type,
    op: &OperatorValue,
    errors: &mut Vec<String>,
) -> Result<Type, ()> {
    let lhs_deref = dereference_type(lhs);
    let rhs_deref = dereference_type(rhs);

    match (lhs_deref, rhs_deref) {
        // Integer bitwise operations
        (Type::Integer, Type::Integer) => Ok(Type::Integer),

        // Supports string operands
        (Type::String, Type::String) => Ok(Type::String),

        // XOR with mixed int/float (convert to int, result is int)
        (Type::Integer, Type::Float) | (Type::Float, Type::Integer) => Ok(Type::Integer),

        _ => {
            errors.push(format!(
                "Invalid operand types for bitwise {}: {} and {}",
                op, lhs, rhs
            ));
            Err(())
        }
    }
}

fn shift_type(lhs: &Type, rhs: &Type, errors: &mut Vec<String>) -> Result<Type, ()> {
    let lhs_deref = dereference_type(lhs);
    let rhs_deref = dereference_type(rhs);

    // Left operand can be int or float
    let lhs_valid = matches!(lhs_deref, Type::Integer | Type::Float);

    // Right operand must be integer (shift amount)
    let rhs_valid = matches!(rhs_deref, Type::Integer);

    if !lhs_valid {
        errors.push(format!("Shift left operand must be numeric, got {}", lhs));
        return Err(());
    }

    if !rhs_valid {
        errors.push(format!("Shift amount must be integer, got {}", rhs));
        return Err(());
    }

    // Result type matches left operand type
    Ok(lhs_deref)
}

fn arithmetic_type(
    lhs: &Type,
    rhs: &Type,
    op: &OperatorValue,
    errors: &mut Vec<String>,
) -> Result<Type, ()> {
    // Dereference types for arithmetic operations
    let lhs_deref = dereference_type(lhs);
    let rhs_deref = dereference_type(rhs);

    match (lhs_deref, rhs_deref) {
        // Integer arithmetic
        (Type::Integer, Type::Integer) => Ok(Type::Integer),

        // Float arithmetic
        (Type::Float, Type::Float) => Ok(Type::Float),

        // Mixed numeric types - result is Float
        (Type::Integer, Type::Float) | (Type::Float, Type::Integer) => {
            // Modulo only works on integers
            if matches!(op, OperatorValue::Modulo) {
                errors.push("Modulo operator requires integer operands".to_string());
                return Err(());
            }
            Ok(Type::Float)
        }

        // String concatenation
        (Type::String, Type::String) if matches!(op, OperatorValue::Plus) => Ok(Type::String),

        // Invalid types
        _ => {
            errors.push(format!(
                "Invalid operand types for {}: {} and {}",
                op, lhs, rhs
            ));
            Err(())
        }
    }
}

fn comparison_type(lhs: &Type, rhs: &Type, errors: &mut Vec<String>) -> Result<Type, ()> {
    if types_compatible(lhs, rhs) {
        Ok(Type::Boolean)
    } else {
        errors.push(format!("Cannot compare types {} and {}", lhs, rhs));
        Err(())
    }
}

fn logical_type(lhs: &Type, rhs: &Type, errors: &mut Vec<String>) -> Result<Type, ()> {
    if lhs == &Type::Boolean && rhs == &Type::Boolean {
        Ok(Type::Boolean)
    } else {
        errors.push(format!(
            "Logical operators require boolean operands, got {} and {}",
            lhs, rhs
        ));
        Err(())
    }
}

/// Dereference a type one level if it's a reference
fn dereference_type(ty: &Type) -> Type {
    match ty {
        Type::Reference(inner) => inner.as_ref().clone(),
        Type::MutableReference(inner) => inner.as_ref().clone(),
        other => other.clone(),
    }
}

pub fn types_compatible(actual: &Type, expected: &Type) -> bool {
    match (actual, expected) {
        // Exact type match
        (a, b) if a == b => true,

        // Integer to Float conversion
        (Type::Integer, Type::Float) => true,

        // Array type compatibility
        (Type::Array(a), Type::Array(b)) => types_compatible(a, b),

        // FixedArray can be assigned to dynamic Array
        (Type::FixedArray(actual_inner, _), Type::Array(expected_inner)) => {
            types_compatible(actual_inner, expected_inner)
        }

        // Dynamic Array cannot be assigned to FixedArray
        (Type::Array(_), Type::FixedArray(_, _)) => false,

        // Immutable reference can bind to immutable reference
        (Type::Reference(a), Type::Reference(b)) => types_compatible(a, b),

        // Mutable reference can bind to mutable reference
        (Type::MutableReference(a), Type::MutableReference(b)) => types_compatible(a, b),

        // Mutable reference can be treated as immutable (coercion)
        (Type::MutableReference(a), Type::Reference(b)) => types_compatible(a, b),

        _ => false,
    }
}
