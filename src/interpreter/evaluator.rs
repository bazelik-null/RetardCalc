use std::ops::Neg;
use crate::interpreter::ast::node::Node;
use crate::interpreter::operators::OperatorType;

/// Evaluates an AST and returns evaluation result.
pub fn eval(node: &Node) -> Result<f64, String> {
    eval_node(node)
}

fn eval_node(node: &Node) -> Result<f64, String> {
    match node {
        // Number
        Node::Number(value) => Ok(*value),

        // Unary expression
        Node::UnaryExpr { op, child } => {
            // Evaluate child node
            let child_value = eval_node(child)?;

            apply_unary_operation(child_value, op)
        }

        // Binary expression
        Node::BinaryExpr { op, lvalue, rvalue } => {
            // Evaluate lvalue
            let left = eval_node(lvalue)?;
            // Evaluate rvalue
            let right = eval_node(rvalue)?;

            apply_binary_operation(left, right, op)
        }
    }
}

fn apply_unary_operation(value: f64, operation: &OperatorType) -> Result<f64, String> {
    match operation {
        OperatorType::Sqrt => Ok(value.sqrt()),
        OperatorType::Ln => Ok(value.ln()),

        OperatorType::Cos => Ok(value.cos()),
        OperatorType::Sin => Ok(value.sin()),
        OperatorType::Tan => Ok(value.tan()),
        OperatorType::Acos => Ok(value.acos()),
        OperatorType::Asin => Ok(value.asin()),
        OperatorType::Atan => Ok(value.atan()),

        OperatorType::Negate => Ok(value.neg()),
        OperatorType::Abs => Ok(value.abs()),
        OperatorType::Round => Ok(value.round()),
        _ => Err(format!("Invalid unary operator: {:?}", operation)),
    }
}

fn apply_binary_operation(left: f64, right: f64, operation: &OperatorType) -> Result<f64, String> {
    match operation {
        OperatorType::Add => Ok(left + right),
        OperatorType::Subtract => Ok(left - right),
        OperatorType::Multiply => Ok(left * right),
        OperatorType::Divide => Ok(left / right),

        OperatorType::Exponent => Ok(left.powf(right)),
        OperatorType::Log => Ok(right.log(left)),

        OperatorType::Modulo => Ok(left.rem_euclid(right)),
        _ => Err(format!("Invalid binary operator: {:?}", operation)),
    }
}
