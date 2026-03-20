// Copyright (c) 2026 bazelik-null

use crate::morsel_interpreter::environment::symbol_table::SymbolTable;
use crate::morsel_interpreter::environment::symbol_table::function_symbol::{
    FunctionParamSymbol, FunctionSymbol,
};
use crate::morsel_interpreter::environment::types::Type;
use crate::morsel_interpreter::environment::value::Value;
use crate::morsel_interpreter::lexer::syntax_operator::{Precedence, SyntaxOperator};
use crate::morsel_interpreter::lexer::token::{LiteralValue, Token};
use crate::morsel_interpreter::parser::ast_node::Node;

pub struct AstBuilder<'a> {
    pub symbol_table: &'a mut SymbolTable,
    tokens: Vec<Token>,
    pos: usize,
}

impl<'a> AstBuilder<'a> {
    pub fn new(symbol_table: &'a mut SymbolTable, tokens: Vec<Token>) -> Self {
        Self {
            symbol_table,
            tokens,
            pos: 0,
        }
    }

    /// Entry point for parser. Parses all statements and returns symbol table with main function.
    pub fn build(mut self) -> Result<&'a SymbolTable, String> {
        self.parse_block()?;
        Ok(self.symbol_table)
    }

    /// Parses function and builds a single block.
    fn parse_block(&mut self) -> Result<Node, String> {
        let mut statements = Vec::new();

        while self.pos < self.tokens.len() && !self.peek_curly() {
            statements.push(self.parse_statement()?);
        }

        Ok(Node::Block(statements))
    }

    /// Parse a single statement (let binding, assignment, or expression).
    fn parse_statement(&mut self) -> Result<Node, String> {
        let node = if self.peek_keyword("let") {
            self.parse_let_binding()?
        } else if self.peek_keyword("fn") {
            self.parse_func_binding()?
        } else if self.is_assignment() {
            self.parse_assignment()?
        } else {
            self.parse_expression()?
        };

        self.consume_semicolon();
        Ok(node)
    }

    /// Parse let binding with optional mutability and type annotation.
    fn parse_let_binding(&mut self) -> Result<Node, String> {
        self.advance(); // consume 'let'

        // Check for mutability
        let mutability = self.consume_keyword("mut");

        // Get variable name
        let name = self.parse_identifier("let")?;

        // Get type annotation (if explicit)
        let type_annotation = self.get_explicit_type(&name)?;

        // Get assignment
        self.expect_operator(SyntaxOperator::Assign)?;
        let value = Box::new(self.parse_expression()?);

        // If no explicit type, infer from the value
        let final_type_annotation = match type_annotation {
            Some(t) => t,
            None => self.infer_type_from_node(&value)?,
        };

        // Register in symbol table
        self.symbol_table
            .variables
            .define(name.clone(), final_type_annotation, mutability)?;

        Ok(Node::LetBinding {
            reference: Box::from(name),
            value,
            type_annotation: final_type_annotation,
        })
    }

    /// Parse function binding with parameters and implementation.
    fn parse_func_binding(&mut self) -> Result<Node, String> {
        self.advance(); // consume 'fn'

        let full_name = self.parse_identifier("fn")?;

        // Extract namespace and function name from identifier
        let (namespace, name) = self.extract_namespace_and_name(&full_name)?;

        // Parse arguments
        self.expect_operator(SyntaxOperator::LParen)?;
        let args = self.parse_func_arguments()?;
        self.expect_operator(SyntaxOperator::RParen)?;

        // Get explicit return type
        let explicit_type = self.get_explicit_type(&name)?;

        // Extract parameter symbols from parsed arguments
        let param_symbols = self.extract_param_symbols(&args);

        let definition_depth = self.symbol_table.depth();

        // Push scope and register parameters
        self.symbol_table.push_scope();
        self.register_func_parameters(&args)?;

        // Parse implementation
        self.expect_operator(SyntaxOperator::CurlyLParen)?;
        let implementation = Box::new(self.parse_block()?);
        self.expect_operator(SyntaxOperator::CurlyRParen)?;

        // Infer and validate return type
        let inferred_return_type = self.infer_type_from_node(&implementation)?;
        let return_type = self.validate_return_type(&name, explicit_type, inferred_return_type)?;

        self.symbol_table.pop_scope();

        // Register function with namespace
        let func_symbol = FunctionSymbol::new(
            name,
            namespace,
            param_symbols,
            return_type,
            definition_depth,
            Some(implementation),
            false,
        );
        self.symbol_table.functions.define(func_symbol)?;

        Ok(Node::FuncBinding())
    }

    /// Parse assignment statement with type validation.
    fn parse_assignment(&mut self) -> Result<Node, String> {
        // Parse identifier
        let name = self.parse_identifier("assignment")?;

        // Look up variable and validate mutability
        let var = self
            .symbol_table
            .variables
            .lookup(&name)
            .ok_or_else(|| format!("Undefined variable: '{}'", name))?
            .clone();

        if !var.mutable {
            return Err(format!("Cannot assign to immutable variable '{}'", name));
        }

        // Advance past '='
        self.advance();

        // Parse the right-hand side expression
        let value = Box::new(self.parse_expression()?);

        // Validate type compatibility
        let value_type = self.infer_type_from_node(&value)?;
        if value_type != var.type_annotation {
            return Err(format!(
                "Type mismatch in assignment to '{}': expected {}, got {}",
                name, var.type_annotation, value_type
            ));
        }

        Ok(Node::Assignment {
            name: Box::from(name),
            value,
        })
    }

    /// Check if current position is an assignment (identifier followed by =).
    fn is_assignment(&self) -> bool {
        matches!(self.peek(), Some(Token::Identifier(_)))
            && matches!(
                self.tokens.get(self.pos + 1).and_then(|t| t.as_operator()),
                Some(&SyntaxOperator::Assign)
            )
    }

    /// Parse an expression with operator precedence.
    fn parse_expression(&mut self) -> Result<Node, String> {
        self.parse_precedence(Precedence::Additive)
    }

    /// Precedence climbing algorithm for binary operators.
    fn parse_precedence(&mut self, min_precedence: Precedence) -> Result<Node, String> {
        let mut left = self.parse_primary()?;

        while let Some(op) = self.peek_operator() {
            let Some(precedence) = op.precedence() else {
                break;
            };

            if precedence < min_precedence {
                break;
            }

            self.advance();

            // Determine minimum precedence for right operand based on associativity
            let next_min = if op.is_right_associative() {
                precedence
            } else {
                precedence.next_higher()
            };

            let right = self.parse_precedence(next_min)?;

            left = Node::Call {
                name: Box::from(op.to_string()),
                args: vec![left, right],
            };
        }

        Ok(left)
    }

    /// Parse primary expression: unary operator or atom.
    fn parse_primary(&mut self) -> Result<Node, String> {
        if let Some(op) = self.peek_operator()
            && op.is_unary()
        {
            return self.parse_unary(op);
        }

        self.parse_atom()
    }

    /// Parse comma-separated list of expressions (for function calls).
    fn parse_arguments(&mut self) -> Result<Vec<Node>, String> {
        self.parse_comma_separated_list(Self::parse_expression)
    }

    /// Parse comma-separated list of function parameters.
    fn parse_func_arguments(&mut self) -> Result<Vec<Node>, String> {
        self.parse_comma_separated_list(Self::parse_func_parameter)
    }

    /// Parse comma-separated lists with custom parser.
    fn parse_comma_separated_list<F>(&mut self, parser: F) -> Result<Vec<Node>, String>
    where
        F: Fn(&mut Self) -> Result<Node, String>,
    {
        if self.peek_operator() == Some(SyntaxOperator::RParen) {
            return Ok(Vec::new());
        }

        let mut items = vec![parser(self)?];

        while self.peek_operator() == Some(SyntaxOperator::Comma) {
            self.advance(); // consume comma
            items.push(parser(self)?);
        }

        Ok(items)
    }

    /// Parse a single function parameter: 'name: type'.
    fn parse_func_parameter(&mut self) -> Result<Node, String> {
        let name = self.parse_identifier("parameter")?;

        // Type annotation is required for parameters
        let type_annotation = match self.get_explicit_type(&name)? {
            Some(t) => t,
            None => {
                return Err(format!(
                    "Expected parameter type annotation at {}",
                    self.pos
                ));
            }
        };

        Ok(Node::LetBinding {
            reference: Box::from(name),
            value: Box::new(Node::Literal(Value::Null)),
            type_annotation,
        })
    }

    /// Parse unary operator: -x, !x, etc.
    fn parse_unary(&mut self, op: SyntaxOperator) -> Result<Node, String> {
        self.advance();
        let child = self.parse_primary()?;

        Ok(Node::Call {
            name: Box::from(op.to_string()),
            args: vec![child],
        })
    }

    /// Parse atomic expression: literal, variable, parenthesized expression, or function call.
    fn parse_atom(&mut self) -> Result<Node, String> {
        match self.peek() {
            Some(Token::Literal(value)) => {
                let node = match value {
                    LiteralValue::Integer(v) => Node::Literal(Value::Integer(*v)),
                    LiteralValue::Float(v) => Node::Literal(Value::Float(*v)),
                    LiteralValue::String(v) => Node::Literal(Value::String(v.parse().unwrap())),
                    LiteralValue::Boolean(v) => Node::Literal(Value::Boolean(*v)),
                };
                self.advance();
                Ok(node)
            }

            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();

                // Check for function call
                if self.peek_operator() == Some(SyntaxOperator::LParen) {
                    self.advance(); // consume '('
                    let args = self.parse_arguments()?;
                    self.expect_operator(SyntaxOperator::RParen)?;

                    // Validate function call
                    let arg_types: Vec<Type> = args
                        .iter()
                        .map(|node| self.infer_type_from_node(node))
                        .collect::<Result<Vec<_>, String>>()?;

                    self.symbol_table
                        .functions
                        .validate_call(&name, &arg_types)?;

                    Ok(Node::Call {
                        name: Box::from(name),
                        args,
                    })
                } else {
                    // Variable reference
                    if !self.symbol_table.variables.exists_in_current_scope(&name) {
                        return Err(format!(
                            "Variable '{}' is not defined at {}",
                            name, self.pos
                        ));
                    }
                    Ok(Node::Reference(Box::from(name)))
                }
            }

            Some(Token::SyntaxToken(SyntaxOperator::LParen)) => {
                self.advance(); // consume '('
                let expr = self.parse_expression()?;
                self.expect_operator(SyntaxOperator::RParen)?;
                Ok(expr)
            }

            _ => self.error_unexpected_token(),
        }
    }

    /// Parse an identifier token and advance position.
    fn parse_identifier(&mut self, context: &str) -> Result<String, String> {
        match self.peek() {
            Some(Token::Identifier(n)) => {
                let name = n.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(format!(
                "Expected identifier in {} at {}",
                context, self.pos
            )),
        }
    }

    /// Consume keyword if present and return success status.
    fn consume_keyword(&mut self, keyword: &str) -> bool {
        if self.peek_keyword(keyword) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Extract parameter symbols from parsed function arguments.
    fn extract_param_symbols(&self, args: &[Node]) -> Vec<FunctionParamSymbol> {
        args.iter()
            .filter_map(|arg| {
                if let Node::LetBinding {
                    reference: name,
                    type_annotation,
                    ..
                } = arg
                {
                    Some(FunctionParamSymbol::new(
                        name.parse().unwrap(),
                        *type_annotation,
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Register function parameters as variables in the current scope.
    fn register_func_parameters(&mut self, args: &[Node]) -> Result<(), String> {
        for arg in args {
            if let Node::LetBinding {
                reference: name,
                type_annotation,
                ..
            } = arg
            {
                self.symbol_table.variables.define(
                    name.clone().parse().unwrap(),
                    *type_annotation,
                    false,
                )?;
            }
        }
        Ok(())
    }

    /// Validate return type matches explicit annotation or use inferred type.
    fn validate_return_type(
        &self,
        func_name: &str,
        explicit_type: Option<Type>,
        inferred_type: Type,
    ) -> Result<Type, String> {
        match explicit_type {
            Some(t) => {
                // Validate explicit type matches inferred type
                if t != inferred_type {
                    return Err(format!(
                        "Return type mismatch in function '{}': declared {}, but body returns {}",
                        func_name, t, inferred_type
                    ));
                }
                Ok(t)
            }
            None => Ok(inferred_type),
        }
    }

    /// Expect a specific operator and advance if found.
    fn expect_operator(&mut self, expected: SyntaxOperator) -> Result<(), String> {
        match self.peek_operator() {
            Some(op) if op == expected => {
                self.advance();
                Ok(())
            }
            Some(op) => Err(format!(
                "Expected '{}', found '{}' at position {}",
                expected, op, self.pos
            )),
            None => Err(format!("Expected '{}', found end of input", expected)),
        }
    }

    /// Parse explicit type annotation if colon is present.
    fn get_explicit_type(&mut self, name: &str) -> Result<Option<Type>, String> {
        if self.peek_operator() == Some(SyntaxOperator::Colon) {
            self.advance(); // consume ':'

            match self.peek() {
                Some(Token::Type(n)) => {
                    let type_annotation = n.parse::<Type>()?;
                    self.advance();
                    Ok(Some(type_annotation))
                }
                _ => Err(format!(
                    "Expected type after ':' for '{}' at {}",
                    name, self.pos
                )),
            }
        } else {
            Ok(None)
        }
    }

    /// Extract namespace and function name from a fully qualified identifier.
    /// Examples:
    /// - "add" -> ("", "add")
    /// - "helpers::add" -> ("helpers", "add")
    /// - "helpers::tools::add" -> ("helpers::tools", "add")
    fn extract_namespace_and_name(&self, full_name: &str) -> Result<(String, String), String> {
        if let Some(last_colon_pos) = full_name.rfind("::") {
            let namespace = full_name[..last_colon_pos].to_string();
            let name = full_name[last_colon_pos + 2..].to_string();

            // Validate namespace format
            if namespace.is_empty() {
                return Err("Namespace cannot be empty before '::'".to_string());
            }
            if name.is_empty() {
                return Err("Function name cannot be empty after '::'".to_string());
            }

            // Validate that namespace and name are valid identifiers
            if !self.is_valid_identifier(&namespace) {
                return Err(format!("Invalid namespace format: '{}'", namespace));
            }
            if !self.is_valid_identifier(&name) {
                return Err(format!("Invalid function name: '{}'", name));
            }

            Ok((namespace, name))
        } else {
            // No namespace, just a function name
            if !self.is_valid_identifier(full_name) {
                return Err(format!("Invalid function name: '{}'", full_name));
            }
            Ok((String::new(), full_name.to_string()))
        }
    }

    /// Check if a string is a valid identifier (can contain letters, digits, underscores).
    fn is_valid_identifier(&self, s: &str) -> bool {
        !s.is_empty()
            && s.chars().all(|c| c.is_alphanumeric() || c == '_')
            && !s.chars().next().unwrap().is_numeric()
    }

    /// Consume semicolon if present.
    fn consume_semicolon(&mut self) {
        if self.peek_operator() == Some(SyntaxOperator::Semicolon) {
            self.advance();
        }
    }

    /// Check if next operator is closing curly brace.
    fn peek_curly(&self) -> bool {
        matches!(self.peek_operator(), Some(SyntaxOperator::CurlyRParen))
    }

    /// Generate error for unexpected token.
    fn error_unexpected_token(&self) -> Result<Node, String> {
        match self.peek() {
            Some(token) => Err(format!(
                "Unexpected token at position {}: {:?}",
                self.pos, token
            )),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    /// Get token at current position.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    /// Get operator from current token if available.
    fn peek_operator(&self) -> Option<SyntaxOperator> {
        self.peek().and_then(|t| t.as_operator().cloned())
    }

    /// Check if current token is a specific keyword.
    fn peek_keyword(&self, keyword: &str) -> bool {
        matches!(self.peek(), Some(Token::Keyword(kw)) if kw == keyword)
    }

    /// Move to next token.
    fn advance(&mut self) {
        self.pos += 1;
    }
}
