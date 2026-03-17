// Copyright (c) 2026 bazelik-null

use crate::interpreter::operators::OperatorType;
use std::fmt;

#[derive(Debug)]
pub enum Node {
    Number(f64),
    UnaryExpr {
        op: OperatorType,
        child: Box<Node>,
    },
    BinaryExpr {
        op: OperatorType,
        lvalue: Box<Node>,
        rvalue: Box<Node>,
    },
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Number(n) => write!(f, "{}", n),
            Node::UnaryExpr { op, child } => {
                write!(f, "{}({})", op, child)
            }
            Node::BinaryExpr { op, lvalue, rvalue } => {
                write!(f, "({} {} {})", lvalue, op, rvalue)
            }
        }
    }
}
