// Copyright (c) 2026 bazelik-null

use std::fmt;

#[derive(Debug, Clone)]
pub enum Node {
    // Literals and references
    Literal(f64),
    Variable(String),

    // Operations (unary, binary, functions)
    Call {
        name: String,
        args: Vec<Node>, // [left, right] for binary, [child] for unary
    },

    // Statements
    Let {
        name: String,
        value: Box<Node>,
    },

    // Blocks
    Block(Vec<Node>),
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ascii_tree = self.tree_string_internal("", true);
        write!(f, "{}", ascii_tree)
    }
}

impl Node {
    fn tree_string_internal(&self, prefix: &str, is_last: bool) -> String {
        let mut result = String::new();

        // Add the current node's connector and content
        let connector = if is_last { "└── " } else { "├── " };
        result.push_str(prefix);
        result.push_str(connector);

        match self {
            Node::Block(statements) => {
                result.push_str("Block\n");

                let extension = if is_last { "    " } else { "│   " };
                let child_prefix = format!("{}{}", prefix, extension);

                // Display all statements
                for (i, stmt) in statements.iter().enumerate() {
                    let is_last_stmt = i == statements.len() - 1;
                    result.push_str(&stmt.tree_string_internal(&child_prefix, is_last_stmt));
                }
            }

            Node::Literal(n) => {
                result.push_str(&format!("Literal({})\n", n));
            }

            Node::Variable(name) => {
                result.push_str(&format!("Variable(\"{}\")\n", name));
            }

            Node::Let { name, value } => {
                result.push_str(&format!("Let(\"{}\")\n", name));

                let extension = if is_last { "    " } else { "│   " };
                let child_prefix = format!("{}{}", prefix, extension);

                result.push_str(&value.tree_string_internal(&child_prefix, true));
            }

            Node::Call { name, args } => {
                result.push_str(&format!("Call(\"{}\")\n", name));

                let extension = if is_last { "    " } else { "│   " };
                let child_prefix = format!("{}{}", prefix, extension);

                // Display all arguments
                for (i, arg) in args.iter().enumerate() {
                    let is_last_arg = i == args.len() - 1;
                    result.push_str(&arg.tree_string_internal(&child_prefix, is_last_arg));
                }
            }
        }

        result
    }

    /// Get the string representation of a node type for debugging
    pub fn node_type(&self) -> &'static str {
        match self {
            Node::Block(_) => "Block",
            Node::Literal(_) => "Literal",
            Node::Variable(_) => "Variable",
            Node::Let { .. } => "Let",
            Node::Call { .. } => "Call",
        }
    }

    /// Check if this node is an atom node (no children)
    pub fn is_atom(&self) -> bool {
        matches!(self, Node::Literal(_) | Node::Variable(_))
    }

    /// Get all child nodes
    pub fn children(&self) -> Vec<&Node> {
        match self {
            Node::Block(statements) => statements.iter().collect(),
            Node::Literal(_) | Node::Variable(_) => vec![],
            Node::Let { value, .. } => vec![value.as_ref()],
            Node::Call { args, .. } => args.iter().collect(),
        }
    }

    /// Get mutable references to all child nodes
    pub fn children_mut(&mut self) -> Vec<&mut Node> {
        match self {
            Node::Block(statements) => statements.iter_mut().collect(),
            Node::Literal(_) | Node::Variable(_) => vec![],
            Node::Let { value, .. } => vec![value.as_mut()],
            Node::Call { args, .. } => args.iter_mut().collect(),
        }
    }

    /// Calculate the depth of the tree
    pub fn depth(&self) -> usize {
        match self {
            Node::Block(statements) => {
                1 + statements
                    .iter()
                    .map(|stmt| stmt.depth())
                    .max()
                    .unwrap_or(0)
            }
            Node::Literal(_) | Node::Variable(_) => 0,
            Node::Let { value, .. } => 1 + value.depth(),
            Node::Call { args, .. } => 1 + args.iter().map(|arg| arg.depth()).max().unwrap_or(0),
        }
    }

    /// Count the total number of nodes in the tree
    pub fn node_count(&self) -> usize {
        1 + self
            .children()
            .iter()
            .map(|child| child.node_count())
            .sum::<usize>()
    }

    /// Get a human-readable tree representation
    pub fn tree_string(&self) -> String {
        self.tree_string_internal("", true)
    }
}
