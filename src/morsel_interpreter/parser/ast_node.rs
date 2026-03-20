// Copyright (c) 2026 bazelik-null

use crate::morsel_interpreter::environment::types::Type;
use crate::morsel_interpreter::environment::value::Value;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Node {
    // Literals
    Literal(Value),
    // Variable references
    Reference(Box<str>),

    // Operations (unary, binary, functions)
    Call {
        name: Box<str>,
        args: Vec<Node>, // [left, right] for binary, [child] for unary
    },

    // Variable binding. Initializes variable
    LetBinding {
        reference: Box<str>,
        value: Box<Node>,
        type_annotation: Type,
    },

    // Function binding
    FuncBinding(),

    // Assignment (x = y)
    Assignment {
        name: Box<str>,
        value: Box<Node>,
    },

    // Blocks
    Block(Vec<Node>),
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tree_string())
    }
}

impl Node {
    /// Get all child nodes
    pub fn children(&self) -> Vec<&Node> {
        match self {
            Node::Block(statements) => statements.iter().collect(),
            Node::Literal(_) | Node::Reference(_) => vec![],
            Node::LetBinding { value, .. } => vec![value.as_ref()],
            Node::Assignment { value, .. } => vec![value.as_ref()],
            Node::Call { args, .. } => args.iter().collect(),
            Node::FuncBinding() => vec![],
        }
    }

    /// Get a human-readable tree representation
    pub fn tree_string(&self) -> String {
        self.format_tree("", true)
    }

    /// Format a single node line without type annotation
    fn format_node_line(&self) -> String {
        match self {
            Node::Block(_) => "Block".to_string(),
            Node::Literal(value) => match value {
                Value::String(_) => format!("Literal(\"{}\")", value),
                _ => format!("Literal({})", value),
            },
            Node::Reference(name) => {
                format!("Variable({})", name)
            }
            Node::LetBinding {
                reference: name, ..
            } => {
                format!("Let({})", name)
            }
            Node::Assignment { name, .. } => {
                format!("Assign({})", name)
            }
            Node::Call { name, .. } => {
                format!("Call({})", name)
            }
            Node::FuncBinding() => "Fn()".to_string(),
        }
    }

    /// Recursively format the tree
    fn format_tree(&self, prefix: &str, is_last: bool) -> String {
        let connector = if is_last { "└── " } else { "├── " };
        let mut result = format!("{}{}{}\n", prefix, connector, self.format_node_line());

        let children = self.children();
        if !children.is_empty() {
            let extension = if is_last { "    " } else { "│   " };
            let child_prefix = format!("{}{}", prefix, extension);

            for (i, child) in children.iter().enumerate() {
                result.push_str(&child.format_tree(&child_prefix, i == children.len() - 1));
            }
        }

        result
    }
}
