use crate::core::compiler::error_handler::CompilerError;
use crate::core::compiler::preprocessor::token::{LiteralValue, OperatorValue};
use crate::core::shared::builtin_func::SysCallId;
use crate::core::shared::types::Type;
use lasso::Spur;

pub struct ParserOutput {
    pub nodes: Vec<Node>,
    pub errors: Vec<CompilerError>,
}

impl Default for ParserOutput {
    fn default() -> Self {
        Self::new()
    }
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
pub struct Parameter {
    pub name: Spur,
    pub type_annotation: Type,
}

#[derive(Debug, Clone)]
pub enum Node {
    // Expressions
    Literal(LiteralValue),
    ArrayLiteral(Vec<Node>),
    Reference {
        value: Box<Node>,
        mutable: bool,
    },
    Dereference(Box<Node>),
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
    FunctionDecl {
        name: Spur,
        params: Vec<Parameter>,
        body: Box<Node>,
        return_type: Option<Type>,
    },
    FunctionCall {
        name: Box<Node>,
        args: Vec<Node>,
    },
    SysCall {
        id: SysCallId,
        args: Vec<Node>,
    },
    ArrayAccess {
        array: Box<Node>,
        index: Box<Node>,
    },
    Return(Option<Box<Node>>),
}
