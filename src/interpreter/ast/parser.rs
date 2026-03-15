use crate::interpreter::ast::node::Node;
use crate::interpreter::operators::OperatorType;
use crate::interpreter::tokenizer::token::Token;

/// A recursive descent parser that converts tokens into an abstract syntax tree (AST).
/// Implements operator precedence parsing to correctly handle order of operations.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Entry point for parsing. Initiates parsing from the lowest precedence level.
    pub fn parse(&mut self) -> Result<Node, String> {
        self.parse_expression()
    }

    /// Handles lowest precedence (addition, subtraction)
    fn parse_expression(&mut self) -> Result<Node, String> {
        // Parse left side
        let mut left = self.parse_term()?;

        // Keep parsing additive operators and their right operands
        while let Some(op) = self.peek_operator() {
            // Stop if the next operator is not additive (e.g., it's multiplicative)
            if !op.is_additive() {
                break;
            }

            // Consume the operator
            self.advance();

            // Parse right operand
            let right = self.parse_term()?;

            // Build binary expression node
            left = Node::BinaryExpr {
                op,
                lvalue: Box::new(left),
                rvalue: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Handles medium precedence (multiplication, division)
    fn parse_term(&mut self) -> Result<Node, String> {
        // Parse left operand
        let mut left = self.parse_unary()?;

        // Keep parsing multiplicative operators and their right operands
        while let Some(op) = self.peek_operator() {
            // Stop if the next operator is not multiplicative
            if !op.is_multiplicative() {
                break;
            }

            // Consume the operator
            self.advance();

            // Parse right operand
            let right = self.parse_exponent()?;

            // Build binary expression node
            left = Node::BinaryExpr {
                op,
                lvalue: Box::new(left),
                rvalue: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Handles high precedence (exponentiation)
    fn parse_exponent(&mut self) -> Result<Node, String> {
        // Parse left operand
        let mut left = self.parse_unary()?;

        // Keep parsing exponentiation operators and their right operands
        while let Some(op) = self.peek_operator() {
            // Stop if the next operator is not exponentiation
            if !op.is_exponentiation() {
                break;
            }

            // Consume the operator
            self.advance();

            // Parse right operand
            let right = self.parse_exponent()?;

            // Build binary expression node
            left = Node::BinaryExpr {
                op,
                lvalue: Box::new(left),
                rvalue: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Handles unary operators ( -(int), +(int) )
    fn parse_unary(&mut self) -> Result<Node, String> {
        // Check if the current token is a unary operator
        if let Some(op) = self.peek_operator()
            && op.is_unary()
        {
            // Consume the unary operator
            self.advance();

            // Recursively parse the operand
            let child = self.parse_unary()?;

            // Build unary expression node
            return Ok(Node::UnaryExpr {
                op,
                child: Box::new(child),
            });
        }

        // Not a unary operator, so parse the next level
        self.parse_primary()
    }

    /// Handles numbers and parentheses
    fn parse_primary(&mut self) -> Result<Node, String> {
        match self.peek() {
            // Handle numbers
            Some(Token::Number(value)) => {
                // Extract the numeric value
                let value = *value;

                // Consume the number token
                self.advance();

                // Return the parsed number node
                Ok(Node::Number(value))
            }
            // Handle brackets
            Some(Token::Operator(OperatorType::LBracket)) => {
                // Consume the opening bracket
                self.advance();

                // Parse the expression inside the brackets
                let expr = self.parse_expression()?;

                // Expect a closing bracket
                match self.peek() {
                    Some(Token::Operator(OperatorType::RBracket)) => {
                        // Consume the closing bracket
                        self.advance();
                        Ok(expr)
                    }
                    _ => Err("Expected closing bracket ')'".to_string()),
                }
            }
            Some(Token::Operator(_)) => {
                Err("Unexpected operator in primary expression".to_string())
            }
            None => Err("Unexpected end of input".to_string()),
        }
    }

    /// Returns a reference to the token at the current position without consuming it.
    /// This allows the parser to look ahead and decide what to do next.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    /// Peeks at the current token and extracts it as an operator.
    fn peek_operator(&self) -> Option<OperatorType> {
        self.peek().and_then(|t| t.as_operator().cloned())
    }

    fn advance(&mut self) {
        self.pos += 1;
    }
}
