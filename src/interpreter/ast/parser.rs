use crate::interpreter::ast::node::Node;
use crate::interpreter::operators::{OperatorType, Precedence};
use crate::interpreter::tokenizer::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

/* Parser flow:
 * parse()                 [Entry point]
 * * parse_precedence()    [Handles binary operators with precedence]
 * * * parse_primary()     [Handles unary/function/atoms]
 * * * * parse_function()  [Function calls like cos(x)]
 * * * * parse_unary()     [Unary operators like -x]
 * * * * parse_atom()      [Numbers and parentheses]
 */
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Entry point for parsing. Initiates parsing from the lowest precedence level.
    pub fn parse(&mut self) -> Result<Node, String> {
        let expr = self.parse_precedence(Precedence::Additive)?;
        if self.pos < self.tokens.len() {
            return Err(format!("Unexpected token at position {}", self.pos));
        }

        Ok(expr)
    }

    fn parse_precedence(&mut self, min_precedence: Precedence) -> Result<Node, String> {
        // Parse left value
        let mut left = self.parse_primary()?;

        // Parse binary operators as long as they have sufficient precedence
        while let Some(op) = self.peek_operator() {
            // Get the precedence of the current operator, or stop if it's not a binary operator
            let Some(precedence) = op.precedence() else {
                break;
            };

            if precedence < min_precedence {
                break;
            }

            // Consume operator
            self.advance();

            // Determine the minimum precedence for the right operand
            let next_min = if op.is_right_associative() {
                precedence
            } else {
                Precedence::Exponent
            };

            // Parse right operands with calculated minimum precedence
            let right = self.parse_precedence(next_min)?;

            // Build binary expression node
            left = Node::BinaryExpr {
                op,
                lvalue: Box::new(left),
                rvalue: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Node, String> {
        if let Some(op) = self.peek_operator() {
            if op.is_function() {
                return self.parse_function(op);
            }
            if op.is_unary() {
                return self.parse_unary(op);
            }
        }

        self.parse_atom()
    }

    fn parse_function(&mut self, op: OperatorType) -> Result<Node, String> {
        // Consume function operator
        self.advance();

        // Check for parenthesis and parse operands inside them
        self.expect(OperatorType::LParen)?;
        let arg = self.parse_precedence(Precedence::Additive)?;
        self.expect(OperatorType::RParen)?;

        // Build unary expression node
        Ok(Node::UnaryExpr {
            op,
            child: Box::new(arg),
        })
    }

    fn parse_unary(&mut self, op: OperatorType) -> Result<Node, String> {
        // Consume unary operator
        self.advance();

        // Parse child
        let child = self.parse_primary()?;

        // Build unary expression node
        Ok(Node::UnaryExpr {
            op,
            child: Box::new(child),
        })
    }

    fn parse_atom(&mut self) -> Result<Node, String> {
        match self.peek() {
            // Parse number
            Some(Token::Number(value)) => {
                let value = *value;
                self.advance();

                Ok(Node::Number(value))
            }

            // Parse parenthesis
            Some(Token::Operator(OperatorType::LParen)) => {
                // Consume left opening bracket, parse operands inside and check for closing bracket
                self.advance();
                let expr = self.parse_precedence(Precedence::Additive)?;
                self.expect(OperatorType::RParen)?;

                Ok(expr)
            }

            Some(Token::Operator(op)) => Err(format!(
                "Unexpected operator '{}' in primary expression",
                op
            )),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn expect(&mut self, expected: OperatorType) -> Result<(), String> {
        match self.peek_operator() {
            Some(op) if op == expected => {
                self.advance();

                Ok(())
            }

            Some(op) => Err(format!("Expected '{}', found '{}'", expected, op)),
            None => Err(format!("Expected '{}', found end of input", expected)),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_operator(&self) -> Option<OperatorType> {
        self.peek().and_then(|t| t.as_operator().cloned())
    }

    fn advance(&mut self) {
        self.pos += 1;
    }
}
