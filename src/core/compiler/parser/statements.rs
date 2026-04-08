use crate::core::compiler::parser::Parser;
use crate::core::compiler::parser::tree::{Node, Parameter};
use crate::core::compiler::preprocessor::token::{
    KeywordValue, LiteralValue, SyntaxValue, TokenType,
};
use crate::core::shared::types::Type;

impl<'a> Parser<'a> {
    pub fn parse_statement_or_expression(&mut self) -> Result<Node, ()> {
        match self.peek_token_type()? {
            TokenType::Keyword(KeywordValue::If) => self.parse_if(),
            TokenType::Keyword(KeywordValue::While) => self.parse_while(),
            TokenType::Keyword(KeywordValue::VariableDecl) => self.parse_var_decl(),
            TokenType::Keyword(KeywordValue::FunctionDecl) => self.parse_function_decl(),
            TokenType::Keyword(KeywordValue::Return) => self.parse_return(),
            TokenType::Syntax(SyntaxValue::LBrace) => self.parse_block(),
            _ => {
                let expr = self.parse_expression(0)?;
                self.consume_terminator();
                Ok(expr)
            }
        }
    }

    fn parse_if(&mut self) -> Result<Node, ()> {
        self.advance();

        self.expect_syntax(SyntaxValue::LParen)?;
        let condition = self.parse_expression(0)?;
        self.expect_syntax(SyntaxValue::RParen)?;

        let then_branch = if self.check_syntax(SyntaxValue::LBrace) {
            self.parse_block()?
        } else {
            self.parse_statement_or_expression()?
        };

        let else_branch = if self.match_keyword(KeywordValue::Else) {
            Some(Box::new(if self.check_syntax(SyntaxValue::LBrace) {
                self.parse_block()?
            } else if self.match_keyword(KeywordValue::If) {
                self.parse_if()? // else if
            } else {
                self.parse_statement_or_expression()?
            }))
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
        let condition = self.parse_expression(0)?;
        self.expect_syntax(SyntaxValue::RParen)?;

        let body = if self.check_syntax(SyntaxValue::LBrace) {
            self.parse_block()?
        } else {
            self.parse_statement_or_expression()?
        };

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
        let value = self.parse_statement_or_expression()?;
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

        let body = self.parse_statement_or_expression()?;

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

    fn parse_params(&mut self) -> Result<Vec<Parameter>, ()> {
        let mut params = Vec::new();

        while !self.check_syntax(SyntaxValue::RParen) {
            let param_name = self.expect_identifier()?;
            self.expect_syntax(SyntaxValue::Colon)?;
            let param_type = self.parse_type()?;

            params.push(Parameter {
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
            let expr = self.parse_expression(0)?;
            self.consume_terminator();
            Ok(Node::Return(Some(Box::new(expr))))
        }
    }

    fn parse_block(&mut self) -> Result<Node, ()> {
        self.expect_syntax(SyntaxValue::LBrace)?;
        let mut statements = Vec::new();

        while !self.is_eof() && !self.check_syntax(SyntaxValue::RBrace) {
            match self.parse_statement_or_expression() {
                Ok(stmt) => {
                    statements.push(stmt);

                    if !self.check_syntax(SyntaxValue::RBrace) {
                        self.consume_terminator();
                    }
                }
                Err(()) => {
                    // Use panic mode to recover within block
                    self.panic_mode_block();
                }
            }
        }

        self.expect_syntax(SyntaxValue::RBrace)?;
        Ok(Node::Block(statements))
    }

    fn parse_type(&mut self) -> Result<Type, ()> {
        // Check for reference prefix
        let is_reference = self.match_keyword(KeywordValue::Reference);

        // Check if mutable
        let mutable = if is_reference {
            self.match_keyword(KeywordValue::Mutable)
        } else {
            false
        };

        // Parse the base type
        let base_type = self.parse_base_type()?;

        // Wrap in Reference if needed
        let result_type = if is_reference {
            match mutable {
                true => Type::MutableReference(Box::new(base_type)),
                false => Type::Reference(Box::new(base_type)),
            }
        } else {
            base_type
        };

        Ok(result_type)
    }

    fn parse_base_type(&mut self) -> Result<Type, ()> {
        let token = self.peek().cloned().ok_or_else(|| {
            self.error_at_current("Unexpected end of input");
        })?;

        match token.token_type {
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
            TokenType::Keyword(KeywordValue::Void) => {
                self.advance();
                Ok(Type::Void)
            }
            TokenType::Syntax(SyntaxValue::LBracket) => {
                self.advance();

                // Check for fixed-size array: [T; size]
                let inner_type = self.parse_type()?;

                if self.match_syntax(SyntaxValue::Semicolon) {
                    // Fixed array
                    let size_token = self.peek().cloned().ok_or_else(|| {
                        self.error_at_current("Expected array size");
                    })?;

                    let size = match size_token.token_type {
                        TokenType::Literal(LiteralValue::Integer(n)) => {
                            self.advance();
                            n as usize
                        }
                        _ => {
                            self.error("Expected integer literal for array size", size_token);
                            return Err(());
                        }
                    };

                    self.expect_syntax(SyntaxValue::RBracket)?;
                    Ok(Type::FixedArray(Box::new(inner_type), size))
                } else {
                    // Dynamic array
                    self.expect_syntax(SyntaxValue::RBracket)?;
                    Ok(Type::Array(Box::new(inner_type)))
                }
            }
            _ => {
                self.error("Expected type", token);
                Err(())
            }
        }
    }
}
