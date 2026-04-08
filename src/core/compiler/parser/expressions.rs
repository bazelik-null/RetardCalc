use crate::core::compiler::parser::Parser;
use crate::core::compiler::parser::tree::Node;
use crate::core::compiler::preprocessor::token::{
    KeywordValue, OperatorValue, SyntaxValue, TokenType,
};
use crate::core::shared::builtin_func::SysCallId;
use std::str::FromStr;

impl<'a> Parser<'a> {
    pub fn parse_expression(&mut self, min_bp: u8) -> Result<Node, ()> {
        let mut lhs = self.parse_prefix()?;
        lhs = self.parse_postfix(lhs)?;

        loop {
            if self.is_eof() {
                break;
            }

            let token = match self.peek() {
                Some(t) => *t,
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

                break;
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
        let token = self.peek().cloned().ok_or_else(|| {
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
            TokenType::Syntax(SyntaxValue::LBracket) => self.parse_array_literal(),
            TokenType::Keyword(KeywordValue::Dereference) => {
                self.advance();

                let rhs = self.parse_prefix()?;

                Ok(Node::Dereference(Box::new(rhs)))
            }
            TokenType::Keyword(KeywordValue::Reference) => {
                self.advance();

                let mutable = self.match_keyword(KeywordValue::Mutable);
                let rhs = self.parse_prefix()?;

                Ok(Node::Reference {
                    value: Box::new(rhs),
                    mutable,
                })
            }
            TokenType::Operator(op) => match op {
                OperatorValue::Plus
                | OperatorValue::Minus
                | OperatorValue::Not
                | OperatorValue::Multiply => {
                    self.advance();

                    let rhs = self.parse_expression(50)?;

                    // Dereference
                    if op == OperatorValue::Multiply {
                        return Ok(Node::Dereference(Box::new(rhs)));
                    }

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

                let expr = self.parse_expression(0)?;
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
            match self.peek_token_type() {
                Ok(TokenType::Syntax(SyntaxValue::LParen)) => {
                    // Function call: func(args)
                    self.advance();
                    let args = self.parse_arguments()?;
                    self.expect_syntax(SyntaxValue::RParen)?;

                    lhs = match lhs {
                        // If call target is an identifier and matches a SysCallId, make SysCall node
                        Node::Identifier(name) => {
                            if let Ok(id) = SysCallId::from_str(self.rodeo.resolve(&name)) {
                                Node::SysCall { id, args }
                            } else {
                                Node::FunctionCall {
                                    name: Box::new(Node::Identifier(name)),
                                    args,
                                }
                            }
                        }
                        // Otherwise, regular function call with arbitrary expression as name
                        _ => Node::FunctionCall {
                            name: Box::new(lhs),
                            args,
                        },
                    };
                }
                Ok(TokenType::Syntax(SyntaxValue::LBracket)) => {
                    // Array indexing: arr[index]
                    self.advance();

                    let index = self.parse_expression(0)?;
                    self.expect_syntax(SyntaxValue::RBracket)?;

                    lhs = Node::ArrayAccess {
                        array: Box::new(lhs),
                        index: Box::new(index),
                    };
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_array_literal(&mut self) -> Result<Node, ()> {
        self.expect_syntax(SyntaxValue::LBracket)?;
        let mut elements = Vec::new();

        // Handle empty arrays: []
        if self.check_syntax(SyntaxValue::RBracket) {
            self.advance();
            return Ok(Node::ArrayLiteral(elements));
        }

        loop {
            if self.is_eof() {
                self.error_at_current("Unexpected EOF in array literal");
                return Err(());
            }

            elements.push(self.parse_expression(0)?);

            if !self.match_syntax(SyntaxValue::Comma) {
                break;
            }

            // Allow trailing comma: [1, 2, 3,]
            if self.check_syntax(SyntaxValue::RBracket) {
                break;
            }
        }

        self.expect_syntax(SyntaxValue::RBracket)?;
        Ok(Node::ArrayLiteral(elements))
    }

    fn parse_arguments(&mut self) -> Result<Vec<Node>, ()> {
        let mut args = Vec::new();

        // Handle empty argument lists: func()
        if self.check_syntax(SyntaxValue::RParen) {
            return Ok(args);
        }

        loop {
            if self.is_eof() {
                self.error_at_current("Unexpected EOF in argument list");
                return Err(());
            }

            args.push(self.parse_expression(0)?);

            if !self.match_syntax(SyntaxValue::Comma) {
                break;
            }

            // Allow trailing comma: func(a, b,)
            if self.check_syntax(SyntaxValue::RParen) {
                break;
            }
        }

        Ok(args)
    }
}
