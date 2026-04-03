use crate::core::compiler::error_handler::CompilerError;
use lasso::Spur;
use std::fmt;
use std::fmt::Formatter;

pub struct LexerOutput {
    pub tokens: Vec<Token>,
    pub errors: Vec<CompilerError>,
}

impl Default for LexerOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl LexerOutput {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: u16,
    pub column: u16,
    pub length: u16,
}

#[derive(Copy, Clone, Debug)]
pub enum TokenType {
    Literal(LiteralValue),   // Literal values (like '1', 'true')
    Operator(OperatorValue), // Operators (like '+', '-')
    Syntax(SyntaxValue),     // Syntax tokens (like '{' '(')
    Identifier(Spur),        // Identifiers (var references, func calls...)
    Keyword(KeywordValue),   // Keywords (like 'let', 'func')
    Eof,                     // End of file marker
}

#[derive(Copy, Clone, Debug)]
pub enum LiteralValue {
    Integer(i32),
    Float(f32),
    Boolean(bool),
    String(Spur),
}

pub enum TokenNumber {
    Integer(i32),
    Float(f32),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OperatorValue {
    // Math operators
    Plus,     // +
    Minus,    // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %
    Power,    // ^
    // Logic operators
    Equal,        // ==
    NotEqual,     // !=
    Not,          // !
    Greater,      // >
    Less,         // <
    GreaterEqual, // >=
    LessEqual,    // <=
    And,          // &&
    Or,           // ||
    Xor,          // ^^
    // Binary operators
    ShiftLeft,  // <<
    ShiftRight, // >>
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SyntaxValue {
    Semicolon, // ;
    Colon,     // :
    Comma,     // .
    Assign,    // =
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    LBracket,  // [
    RBracket,  // ]
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum KeywordValue {
    // Declarations
    FunctionDecl, // func
    VariableDecl, // let
    Mutable,      // mut
    // Control flow
    If,     // if
    Else,   // else
    Return, // return
    // Loops
    While, // while
    For,   // for
    // Types
    Integer, // int
    Float,   // float
    Boolean, // bool
    String,  // string
    Void,    // void
    // References
    Reference,   // ref
    Dereference, // deref
}

impl Token {
    pub fn new(token_type: TokenType, line: u16, column: u16, length: u16) -> Self {
        Self {
            token_type,
            line,
            column,
            length,
        }
    }
}

impl fmt::Display for OperatorValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let op = match self {
            OperatorValue::Plus => "+",
            OperatorValue::Minus => "-",
            OperatorValue::Multiply => "*",
            OperatorValue::Divide => "/",
            OperatorValue::Modulo => "%",
            OperatorValue::Power => "^",
            OperatorValue::Equal => "==",
            OperatorValue::NotEqual => "!=",
            OperatorValue::Not => "!",
            OperatorValue::Greater => ">",
            OperatorValue::Less => "<",
            OperatorValue::GreaterEqual => ">=",
            OperatorValue::LessEqual => "<=",
            OperatorValue::And => "&&",
            OperatorValue::Or => "||",
            OperatorValue::Xor => "^^",
            OperatorValue::ShiftLeft => "<<",
            OperatorValue::ShiftRight => ">>",
        };

        write!(f, "{}", op)
    }
}
