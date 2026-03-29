#![allow(clippy::result_unit_err)]
pub mod analyzer;
mod expressions;
mod statements;
pub mod symbol;
pub mod tree;
pub mod type_inference;

use crate::core::compiler::error_handler::CompilerError;
use crate::core::compiler::parser::analyzer::SemanticAnalyzer;
use crate::core::compiler::parser::tree::ParserOutput;
use crate::core::compiler::preprocessor::token::{
    KeywordValue, LexerOutput, OperatorValue, SyntaxValue, Token, TokenType,
};
use crate::core::compiler::source::SourceCode;
use lasso::{Rodeo, Spur};

pub struct Parser<'a> {
    source_code: &'a SourceCode,
    rodeo: &'a Rodeo,
    tokens: Vec<Token>,
    pos: usize,
    output: ParserOutput,
}

impl<'a> Parser<'a> {
    pub fn new(lexer_output: LexerOutput, source_code: &'a SourceCode, rodeo: &'a Rodeo) -> Self {
        let output = ParserOutput::new();
        Self {
            source_code,
            rodeo,
            tokens: lexer_output.tokens,
            pos: 0,
            output,
        }
    }

    pub fn parse(mut self) -> ParserOutput {
        while !self.is_eof() {
            match self.parse_statement_or_expression() {
                Ok(node) => self.output.nodes.push(node),
                Err(()) => {
                    // Enter panic mode
                    self.panic_mode();
                }
            }
        }

        // Run semantic analysis
        let mut analyzer = SemanticAnalyzer::new(self.rodeo);
        match analyzer.analyze(&mut self.output.nodes) {
            Ok(()) => {
                // Semantic analysis passed
            }
            Err(semantic_errors) => {
                // Convert semantic errors to codegen errors
                for sem_err in semantic_errors {
                    let compiler_err = CompilerError::new(
                        sem_err,
                        "Analyzer".to_string(),
                        0,
                        0,
                        0,
                        None,
                        Some(self.source_code.filename.clone()),
                    );
                    self.output.errors.push(compiler_err);
                }
            }
        }

        self.output
    }

    //
    // Helpers
    //

    fn peek_token_type(&self) -> Result<TokenType, ()> {
        self.peek().map(|t| t.token_type).ok_or(())
    }

    fn check_syntax(&self, syntax: SyntaxValue) -> bool {
        matches!(
            self.peek().map(|t| &t.token_type),
            Some(TokenType::Syntax(s)) if s == &syntax
        )
    }

    fn match_syntax(&mut self, syntax: SyntaxValue) -> bool {
        if self.check_syntax(syntax) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_keyword(&mut self, keyword: KeywordValue) -> bool {
        if matches!(
            self.peek().map(|t| &t.token_type),
            Some(TokenType::Keyword(s)) if s == &keyword
        ) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn is_statement_terminator(&self) -> bool {
        matches!(
            self.peek().map(|t| &t.token_type),
            Some(TokenType::Syntax(
                SyntaxValue::Semicolon | SyntaxValue::RBrace
            )) | None
        )
    }

    fn consume_terminator(&mut self) {
        if matches!(
            self.peek().map(|t| &t.token_type),
            Some(TokenType::Syntax(SyntaxValue::Semicolon))
        ) {
            self.advance();
        }
    }

    fn expect_identifier(&mut self) -> Result<Spur, ()> {
        let token = self.peek().cloned().ok_or_else(|| {
            self.error_at_current("Expected identifier, found EOF");
        })?;

        if let TokenType::Identifier(spur) = token.token_type {
            self.advance();
            Ok(spur)
        } else {
            self.error("Expected identifier", token);
            Err(())
        }
    }

    fn expect_syntax(&mut self, expected: SyntaxValue) -> Result<(), ()> {
        let token = self.peek().cloned().ok_or_else(|| {
            self.error_at_current(&format!("Expected {:?}, found EOF", expected));
        })?;

        if matches!(&token.token_type, TokenType::Syntax(s) if s == &expected) {
            self.advance();
            Ok(())
        } else {
            self.error(&format!("Expected {:?}", expected), token);
            Err(())
        }
    }

    fn error_at_current(&mut self, message: &str) {
        let token = self.peek().cloned().unwrap_or_else(|| self.get_eof_token());
        self.error(message, token);
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
            || matches!(
                self.peek().map(|t| &t.token_type),
                Some(TokenType::Eof) | None
            )
    }

    fn error(&mut self, message: &str, token: Token) {
        let err = CompilerError::new(
            message.to_string(),
            "Parser".to_string(),
            token.line,
            token.column,
            token.length,
            self.source_code.get_line(token.line),
            Some(self.source_code.filename.clone()),
        );
        self.output.errors.push(err);
    }

    /// Get last token for error reporting
    fn get_eof_token(&self) -> Token {
        self.tokens.last().cloned().unwrap_or(Token {
            token_type: TokenType::Eof,
            line: 0,
            column: 0,
            length: 0,
        })
    }

    /// Returns Some((left_bp, right_bp)) for known operators
    /// Right associative operators should have rbp < lbp.
    fn get_binding_power(op: OperatorValue) -> Option<(u8, u8)> {
        Some(match op {
            OperatorValue::Or => (1, 2),
            OperatorValue::And => (3, 4),
            OperatorValue::Equal | OperatorValue::NotEqual => (5, 6),
            OperatorValue::Greater
            | OperatorValue::Less
            | OperatorValue::GreaterEqual
            | OperatorValue::LessEqual => (7, 8),
            OperatorValue::Plus | OperatorValue::Minus => (9, 10),
            OperatorValue::Multiply | OperatorValue::Divide | OperatorValue::Modulo => (11, 12),
            OperatorValue::Power => (14, 13), // Right associative
            _ => return None,
        })
    }

    //
    // Panic Mode Recovery
    //

    /// Synchronize the token stream by skipping tokens until a safe recovery point is found
    fn panic_mode(&mut self) {
        let start_pos = self.pos;

        // Skip tokens until we find a synchronization point
        while !self.is_eof() {
            match self.peek_token_type() {
                Ok(token_type) => {
                    match token_type {
                        // Statement terminators - safe to resume
                        TokenType::Syntax(SyntaxValue::Semicolon | SyntaxValue::RBrace) => {
                            self.consume_terminator();
                            return;
                        }
                        // Keywords that start statements - safe to resume
                        TokenType::Keyword(
                            KeywordValue::If
                            | KeywordValue::While
                            | KeywordValue::VariableDecl
                            | KeywordValue::FunctionDecl
                            | KeywordValue::Return,
                        ) => {
                            return;
                        }
                        // Block start - safe to resume
                        TokenType::Syntax(SyntaxValue::LBrace) => {
                            return;
                        }
                        _ => self.advance(),
                    }
                }
                Err(()) => break,
            }
        }

        // If we skipped tokens without finding a sync point, log which tokens we skipped
        if self.pos > start_pos {
            let skipped_count = self.pos - start_pos;
            self.error_at_current(&format!(
                "Skipped {} tokens to recover from parse error",
                skipped_count
            ));
        }
    }

    /// Synchronize the token stream inside block by skipping tokens until a safe recovery point is found
    fn panic_mode_block(&mut self) {
        while !self.is_eof() && !self.check_syntax(SyntaxValue::RBrace) {
            match self.peek_token_type() {
                Ok(token_type) => match token_type {
                    TokenType::Syntax(SyntaxValue::Semicolon) => {
                        self.advance();
                        return;
                    }
                    TokenType::Keyword(
                        KeywordValue::If
                        | KeywordValue::While
                        | KeywordValue::VariableDecl
                        | KeywordValue::FunctionDecl
                        | KeywordValue::Return,
                    ) => {
                        return;
                    }
                    TokenType::Syntax(SyntaxValue::LBrace) => {
                        return;
                    }
                    _ => self.advance(),
                },
                Err(()) => break,
            }
        }
    }
}
