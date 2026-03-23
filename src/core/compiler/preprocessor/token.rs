use crate::core::compiler::error_handler::CompilerError;
use lasso::{Rodeo, Spur};

pub struct LexerOutput {
    pub rodeo: Rodeo,
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
            rodeo: Rodeo::new(),
            errors: Vec::new(),
        }
    }

    pub fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub line: u16,
    pub column: u16,
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

pub enum Number {
    Integer(i32),
    Float(f32),
}

#[derive(Copy, Clone, Debug)]
pub enum OperatorValue {
    // Math operators
    Plus,     // +
    Minus,    // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %
    Power,    // ^
    // Logic operators
    Equal,              // ==
    NotEqual,           // !=
    Not,                // !
    GreaterThan,        // >
    LessThan,           // <
    GreaterThanOrEqual, // >=
    LessThanOrEqual,    // <=
    And,                // &&
    Or,                 // ||
}

#[derive(Copy, Clone, Debug)]
pub enum SyntaxValue {
    Semicolon, // ;
    Colon,     // :
    Comma,     // .
    Assign,    // =
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
}

#[derive(Copy, Clone, Debug)]
pub enum KeywordValue {
    FunctionDecl, // func
    VariableDecl, // let
    Mutable,      // mut
    If,           // if
    Else,         // else
    While,        // while
    For,          // for
}

impl Token {
    pub fn new(token_type: TokenType, line: u16, column: u16) -> Self {
        Self {
            token_type,
            line,
            column,
        }
    }
}
