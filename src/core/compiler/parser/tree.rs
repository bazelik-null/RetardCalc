use crate::core::compiler::error_handler::CompilerError;
use crate::core::compiler::preprocessor::token::{LiteralValue, OperatorValue};
use lasso::{Rodeo, Spur};
use std::fmt;
use std::fmt::Formatter;

pub struct ParserOutput {
    pub nodes: Vec<Node>,
    pub errors: Vec<CompilerError>,
}

impl ParserOutput {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            errors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Integer,
    Float,
    Boolean,
    String,
    Array(Box<Type>),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Type::Integer => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Boolean => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Array(inner) => write!(f, "[{}]", inner),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    // Expressions
    Literal(LiteralValue),
    Identifier(Spur),
    Unary {
        op: OperatorValue,
        rhs: Box<Node>,
    },
    Binary {
        lhs: Box<Node>,
        op: OperatorValue,
        rhs: Box<Node>,
    },
    Assignment {
        target: Box<Node>,
        value: Box<Node>,
    },

    // Statements
    Block(Vec<Node>),
    If {
        condition: Box<Node>,
        then_branch: Box<Node>,
        else_branch: Option<Box<Node>>,
    },
    While {
        condition: Box<Node>,
        body: Box<Node>,
    },
    VariableDecl {
        name: Spur,
        mutable: bool,
        type_annotation: Option<Type>,
        value: Box<Node>,
    },
    ParamDecl {
        name: Spur,
        type_annotation: Type,
    },
    FunctionDecl {
        name: Spur,
        params: Vec<Node>, // ParamDecl nodes
        body: Box<Node>,
        return_type: Option<Type>,
    },
    FunctionCall {
        name: Box<Node>,
        args: Vec<Node>,
    },
    Return(Option<Box<Node>>),
}

impl Node {
    pub fn print(&self, rodeo: &Rodeo) -> String {
        self.print_tree(rodeo, "", true)
    }

    fn print_tree(&self, rodeo: &Rodeo, prefix: &str, is_last: bool) -> String {
        let connector = if is_last { "└── " } else { "├── " };
        let extension = if is_last { "    " } else { "│   " };

        let mut result = format!("{}{}{}\n", prefix, connector, self.node_label(rodeo));

        match self {
            Node::Literal(_) | Node::Identifier(_) | Node::Return(None) => {
                // Leaf nodes, no children to print
            }

            Node::Unary { op, rhs } => {
                result.push_str(&format!("{}{}op: {}\n", prefix, extension, op));
                result.push_str(&rhs.print_tree(rodeo, &format!("{}{}", prefix, extension), true));
            }

            Node::Binary { lhs, op, rhs } => {
                result.push_str(&format!("{}{}op: {}\n", prefix, extension, op));
                result.push_str(&lhs.print_tree(rodeo, &format!("{}{}", prefix, extension), false));
                result.push_str(&rhs.print_tree(rodeo, &format!("{}{}", prefix, extension), true));
            }

            Node::Assignment { target, value } => {
                result.push_str(&target.print_tree(
                    rodeo,
                    &format!("{}{}", prefix, extension),
                    false,
                ));
                result.push_str(&value.print_tree(
                    rodeo,
                    &format!("{}{}", prefix, extension),
                    true,
                ));
            }

            Node::Block(stmts) => {
                for (i, stmt) in stmts.iter().enumerate() {
                    let is_last_stmt = i == stmts.len() - 1;
                    result.push_str(&stmt.print_tree(
                        rodeo,
                        &format!("{}{}", prefix, extension),
                        is_last_stmt,
                    ));
                }
            }

            Node::If {
                condition,
                then_branch,
                else_branch,
            } => {
                result.push_str(&condition.print_tree(
                    rodeo,
                    &format!("{}{}", prefix, extension),
                    false,
                ));
                result.push_str(&then_branch.print_tree(
                    rodeo,
                    &format!("{}{}", prefix, extension),
                    else_branch.is_none(),
                ));
                if let Some(else_b) = else_branch {
                    result.push_str(&else_b.print_tree(
                        rodeo,
                        &format!("{}{}", prefix, extension),
                        true,
                    ));
                }
            }

            Node::While { condition, body } => {
                result.push_str(&condition.print_tree(
                    rodeo,
                    &format!("{}{}", prefix, extension),
                    false,
                ));
                result.push_str(&body.print_tree(rodeo, &format!("{}{}", prefix, extension), true));
            }

            Node::VariableDecl {
                name: _name,
                mutable,
                type_annotation,
                value,
            } => {
                if *mutable {
                    result.push_str(&format!("{}{}mutable: true\n", prefix, extension));
                }
                if let Some(ty) = type_annotation {
                    result.push_str(&format!("{}{}type: {}\n", prefix, extension, ty));
                }
                result.push_str(&value.print_tree(
                    rodeo,
                    &format!("{}{}", prefix, extension),
                    true,
                ));
            }

            Node::ParamDecl {
                name: _name,
                type_annotation,
            } => {
                result.push_str(&format!(
                    "{}{}type: {}\n",
                    prefix, extension, type_annotation
                ));
            }

            Node::FunctionDecl {
                name: _name,
                params,
                body,
                return_type,
            } => {
                if !params.is_empty() {
                    result.push_str(&format!("{}{}params:\n", prefix, extension));
                    for (i, param) in params.iter().enumerate() {
                        let is_last_param = i == params.len() - 1;
                        result.push_str(&param.print_tree(
                            rodeo,
                            &format!("{}{}  ", prefix, extension),
                            is_last_param,
                        ));
                    }
                }
                if let Some(ret_ty) = return_type {
                    result.push_str(&format!("{}{}return_type: {}\n", prefix, extension, ret_ty));
                }
                result.push_str(&body.print_tree(rodeo, &format!("{}{}", prefix, extension), true));
            }

            Node::FunctionCall { name, args } => {
                result.push_str(&name.print_tree(
                    rodeo,
                    &format!("{}{}", prefix, extension),
                    args.is_empty(),
                ));
                for (i, arg) in args.iter().enumerate() {
                    let is_last_arg = i == args.len() - 1;
                    result.push_str(&arg.print_tree(
                        rodeo,
                        &format!("{}{}", prefix, extension),
                        is_last_arg,
                    ));
                }
            }

            Node::Return(Some(expr)) => {
                result.push_str(&expr.print_tree(rodeo, &format!("{}{}", prefix, extension), true));
            }
        }

        result
    }

    fn node_label(&self, rodeo: &Rodeo) -> String {
        match self {
            Node::Literal(lit) => match lit {
                LiteralValue::Integer(i) => format!("Literal: {}", i),
                LiteralValue::Float(f) => format!("Literal: {}", f),
                LiteralValue::Boolean(b) => format!("Literal: {}", b),
                LiteralValue::String(spur) => format!("Literal: \"{}\"", rodeo.resolve(spur)),
            },
            Node::Identifier(spur) => format!("Identifier: {}", rodeo.resolve(spur)),
            Node::Unary { .. } => "Unary".to_string(),
            Node::Binary { .. } => "Binary".to_string(),
            Node::Assignment { .. } => "Assignment".to_string(),
            Node::Block(_) => "Block".to_string(),
            Node::If { .. } => "If".to_string(),
            Node::While { .. } => "While".to_string(),
            Node::VariableDecl { name, .. } => {
                format!("VariableDecl: {}", rodeo.resolve(name))
            }
            Node::ParamDecl { name, .. } => {
                format!("ParamDecl: {}", rodeo.resolve(name))
            }
            Node::FunctionDecl { name, .. } => {
                format!("FunctionDecl: {}", rodeo.resolve(name))
            }
            Node::FunctionCall { .. } => "FunctionCall".to_string(),
            Node::Return(_) => "Return".to_string(),
        }
    }
}
