use crate::interpreter::ast::node::Node;
use crate::interpreter::operators::OperatorType;

/// Evaluates an AST and returns the result
pub fn eval(node: &Node) -> Result<f64, String> {
    eval_node(node)
}

fn eval_node(node: &Node) -> Result<f64, String> {
    match node {
        // Number
        Node::Number(value) => Ok(*value),

        // Unary expression
        Node::UnaryExpr { op, child } => {
            let value = eval_node(child)?;

            apply_unary(*op, value)
        }

        // Binary expression
        Node::BinaryExpr { op, lvalue, rvalue } => {
            // Evaluate lvalue
            let left = eval_node(lvalue)?;
            // Evaluate rvalue
            let right = eval_node(rvalue)?;

            apply_binary(*op, left, right)
        }
    }
}

/// Applies a unary operation
fn apply_unary(op: OperatorType, value: f64) -> Result<f64, String> {
    match op {
        // Arithmetic
        OperatorType::Negate => Ok(-value),
        OperatorType::Abs => Ok(value.abs()),
        OperatorType::Round => Ok(value.round()),

        // Exponential/Logarithmic
        OperatorType::Sqrt => Ok(value.sqrt()),
        OperatorType::Ln => Ok(value.ln()),

        // Trigonometric
        OperatorType::Sin => Ok(value.sin()),
        OperatorType::Cos => Ok(value.cos()),
        OperatorType::Tan => Ok(value.tan()),
        OperatorType::Asin => Ok(value.asin()),
        OperatorType::Acos => Ok(value.acos()),
        OperatorType::Atan => Ok(value.atan()),

        _ => Err(format!("'{}' is not a unary operator", op)),
    }
}

/// Applies a binary operation
fn apply_binary(op: OperatorType, left: f64, right: f64) -> Result<f64, String> {
    match op {
        OperatorType::Add => Ok(left + right),
        OperatorType::Subtract => Ok(left - right),
        OperatorType::Multiply => Ok(left * right),
        OperatorType::Divide => Ok(left / right),

        OperatorType::Exponent => Ok(left.powf(right)),
        OperatorType::Log => Ok(right.log(left)),

        OperatorType::Modulo => Ok(left.rem_euclid(right)),
        _ => Err(format!("'{}' is not a binary operator", op)),
    }
}
