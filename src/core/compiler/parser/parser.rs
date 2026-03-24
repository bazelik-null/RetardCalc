use crate::core::compiler::error_handler::CompilerError;
use crate::core::compiler::parser::tree::{Node, ParserOutput, Type};
use crate::core::compiler::preprocessor::token::{
    KeywordValue, LexerOutput, OperatorValue, SyntaxValue, Token, TokenType,
};
use crate::core::compiler::source::SourceCode;
use lasso::Spur;

pub struct Parser<'a> {
    source_code: &'a SourceCode,
    tokens: Vec<Token>,
    pos: usize,
    output: ParserOutput,
}

impl<'a> Parser<'a> {
    pub fn new(lexer_output: LexerOutput, source_code: &'a SourceCode) -> Self {
        let output = ParserOutput::new();
        Self {
            source_code,
            tokens: lexer_output.tokens,
            pos: 0,
            output,
        }
    }

    pub fn parse(mut self) -> ParserOutput {
        while !self.is_eof() {
            match self.parse_statement() {
                Ok(node) => self.output.nodes.push(node),
                Err(()) => {
                    // Enter panic mode
                    self.panic_mode();
                }
            }
        }
        self.output
    }

    //
    // Statements
    //

    fn parse_statement(&mut self) -> Result<Node, ()> {
        match self.peek_token_type()? {
            TokenType::Keyword(KeywordValue::If) => self.parse_if(),
            TokenType::Keyword(KeywordValue::While) => self.parse_while(),
            TokenType::Keyword(KeywordValue::VariableDecl) => self.parse_var_decl(),
            TokenType::Keyword(KeywordValue::FunctionDecl) => self.parse_function_decl(),
            TokenType::Keyword(KeywordValue::Return) => self.parse_return(),
            TokenType::Syntax(SyntaxValue::LBrace) => self.parse_block(),
            _ => {
                let expr = self.parse_full_expression()?;
                self.consume_terminator();
                Ok(expr)
            }
        }
    }

    fn parse_if(&mut self) -> Result<Node, ()> {
        self.advance();

        self.expect_syntax(SyntaxValue::LParen)?;
        let condition = self.parse_full_expression()?;
        self.expect_syntax(SyntaxValue::RParen)?;

        let then_branch = self.parse_statement()?;

        let else_branch = if self.match_keyword(KeywordValue::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Node::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn parse_while(&mut self) -> Result<Node, ()> {
        self.advance();

        self.expect_syntax(SyntaxValue::LParen)?;
        let condition = self.parse_full_expression()?;
        self.expect_syntax(SyntaxValue::RParen)?;

        let body = self.parse_statement()?;

        Ok(Node::While {
            condition: Box::new(condition),
            body: Box::new(body),
        })
    }

    fn parse_var_decl(&mut self) -> Result<Node, ()> {
        self.advance();
        let mutable = self.match_keyword(KeywordValue::Mutable);
        let name = self.expect_identifier()?;

        let type_annotation = if self.match_syntax(SyntaxValue::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect_syntax(SyntaxValue::Assign)?;
        let value = self.parse_full_expression()?;
        self.consume_terminator();

        Ok(Node::VariableDecl {
            name,
            mutable,
            type_annotation,
            value: Box::new(value),
        })
    }

    fn parse_function_decl(&mut self) -> Result<Node, ()> {
        self.advance();

        let name = self.expect_identifier()?;

        self.expect_syntax(SyntaxValue::LParen)?;
        let params = self.parse_params()?;
        self.expect_syntax(SyntaxValue::RParen)?;

        let return_type = if self.match_syntax(SyntaxValue::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_statement()?;

        // Wrap in a Block
        let body = match body {
            Node::Block(_) => body,
            _ => Node::Block(vec![body]),
        };

        Ok(Node::FunctionDecl {
            name,
            params,
            body: Box::new(body),
            return_type,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Node>, ()> {
        let mut params = Vec::new();

        while !self.check_syntax(SyntaxValue::RParen) {
            let param_name = self.expect_identifier()?;
            self.expect_syntax(SyntaxValue::Colon)?;
            let param_type = self.parse_type()?;

            params.push(Node::ParamDecl {
                name: param_name,
                type_annotation: param_type,
            });

            if !self.match_syntax(SyntaxValue::Comma) {
                break;
            }
        }

        Ok(params)
    }

    fn parse_return(&mut self) -> Result<Node, ()> {
        self.advance();

        if self.is_statement_terminator() {
            self.consume_terminator();
            Ok(Node::Return(None))
        } else {
            let expr = self.parse_full_expression()?;
            self.consume_terminator();
            Ok(Node::Return(Some(Box::new(expr))))
        }
    }

    fn parse_block(&mut self) -> Result<Node, ()> {
        self.expect_syntax(SyntaxValue::LBrace)?;
        let mut statements = Vec::new();

        while !self.is_eof() && !self.check_syntax(SyntaxValue::RBrace) {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(()) => {
                    // Use panic mode to recover within block
                    self.panic_mode_block();
                }
            }
        }

        self.expect_syntax(SyntaxValue::RBrace)?;
        Ok(Node::Block(statements))
    }

    /// Panic mode specifically for within blocks - stops at closing brace
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

    fn parse_type(&mut self) -> Result<Type, ()> {
        match self.peek_token_type()? {
            TokenType::Keyword(KeywordValue::Integer) => {
                self.advance();
                Ok(Type::Integer)
            }
            TokenType::Keyword(KeywordValue::Float) => {
                self.advance();
                Ok(Type::Float)
            }
            TokenType::Keyword(KeywordValue::Boolean) => {
                self.advance();
                Ok(Type::Boolean)
            }
            TokenType::Keyword(KeywordValue::String) => {
                self.advance();
                Ok(Type::String)
            }
            TokenType::Syntax(SyntaxValue::LBracket) => {
                self.advance();
                let inner_type = self.parse_type()?;
                self.expect_syntax(SyntaxValue::RBracket)?;
                Ok(Type::Array(Box::new(inner_type)))
            }
            _ => {
                let token = self.peek_full().cloned().ok_or(())?;
                self.error(
                    &format!("Expected type, found {:?}", token.token_type),
                    token,
                );
                Err(())
            }
        }
    }

    //
    // Expressions
    //

    fn parse_full_expression(&mut self) -> Result<Node, ()> {
        self.parse_expression(0)
    }

    fn parse_expression(&mut self, min_bp: u8) -> Result<Node, ()> {
        let mut lhs = self.parse_prefix()?;
        lhs = self.parse_postfix(lhs)?;

        loop {
            if self.is_eof() {
                break;
            }

            let token = match self.peek_full() {
                Some(t) => t,
                None => break,
            };

            // Check for assignment
            if matches!(token.token_type, TokenType::Syntax(SyntaxValue::Assign)) {
                self.advance();
                let rhs = self.parse_expression(1)?;
                lhs = Node::Assignment {
                    target: Box::new(lhs),
                    value: Box::new(rhs),
                };
                continue;
            }

            let op = match token.token_type {
                TokenType::Operator(op) => op,
                _ => break,
            };

            let (lbp, rbp) = Self::get_binding_power(op).ok_or(())?;

            if lbp < min_bp {
                break;
            }

            self.advance();
            let rhs = self.parse_expression(rbp)?;
            lhs = Node::Binary {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Node, ()> {
        let token = self.peek_full().cloned().ok_or_else(|| {
            self.error_at_current("Unexpected end of input");
        })?;

        match token.token_type {
            TokenType::Literal(lit) => {
                self.advance();
                Ok(Node::Literal(lit))
            }
            TokenType::Identifier(spur) => {
                self.advance();
                Ok(Node::Identifier(spur))
            }
            TokenType::Operator(op) => match op {
                OperatorValue::Plus | OperatorValue::Minus | OperatorValue::Not => {
                    self.advance();
                    let rhs = self.parse_expression(100)?;
                    Ok(Node::Unary {
                        op,
                        rhs: Box::new(rhs),
                    })
                }
                _ => {
                    self.advance();
                    self.error("Unexpected token", token);
                    Err(())
                }
            },
            TokenType::Syntax(SyntaxValue::LParen) => {
                self.advance();
                let expr = self.parse_full_expression()?;
                self.expect_syntax(SyntaxValue::RParen)?;
                Ok(expr)
            }
            _ => {
                self.advance();
                self.error("Unexpected token", token);
                Err(())
            }
        }
    }

    fn parse_postfix(&mut self, mut lhs: Node) -> Result<Node, ()> {
        loop {
            if self.check_syntax(SyntaxValue::LParen) {
                self.advance();
                let args = self.parse_arguments()?;
                self.expect_syntax(SyntaxValue::RParen)?;
                lhs = Node::FunctionCall {
                    name: Box::new(lhs),
                    args,
                };
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_arguments(&mut self) -> Result<Vec<Node>, ()> {
        let mut args = Vec::new();

        while !self.check_syntax(SyntaxValue::RParen) {
            args.push(self.parse_full_expression()?);
            if !self.match_syntax(SyntaxValue::Comma) {
                break;
            }
        }

        Ok(args)
    }

    //
    // Helpers
    //

    fn peek_token_type(&self) -> Result<TokenType, ()> {
        self.peek_full().map(|t| t.token_type).ok_or(())
    }

    fn check_syntax(&self, syntax: SyntaxValue) -> bool {
        matches!(
            self.peek_full().map(|t| &t.token_type),
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
            self.peek_full().map(|t| &t.token_type),
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
            self.peek_full().map(|t| &t.token_type),
            Some(TokenType::Syntax(
                SyntaxValue::Semicolon | SyntaxValue::RBrace
            )) | None
        )
    }

    fn consume_terminator(&mut self) {
        if matches!(
            self.peek_full().map(|t| &t.token_type),
            Some(TokenType::Syntax(SyntaxValue::Semicolon))
        ) {
            self.advance();
        }
    }

    fn expect_identifier(&mut self) -> Result<Spur, ()> {
        let token = self.peek_full().cloned().ok_or_else(|| {
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
        let token = self.peek_full().cloned().ok_or_else(|| {
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
        let token = self
            .peek_full()
            .cloned()
            .unwrap_or_else(|| self.get_eof_token());
        self.error(message, token);
    }

    fn peek_full(&self) -> Option<&Token> {
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
                self.peek_full().map(|t| &t.token_type),
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
            OperatorValue::GreaterThan
            | OperatorValue::LessThan
            | OperatorValue::GreaterThanOrEqual
            | OperatorValue::LessThanOrEqual => (7, 8),
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
}
