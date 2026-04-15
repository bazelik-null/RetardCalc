use crate::core::compiler::error_handler::CompilerError;
use crate::core::compiler::preprocessor::token::{
    KeywordValue, LexerOutput, LiteralValue, OperatorValue, SyntaxValue, Token, TokenNumber,
    TokenType,
};
use crate::core::compiler::source::SourceCode;
use lasso::{Rodeo, Spur};

pub struct Lexer<'a> {
    source_code: &'a SourceCode,
    pos: usize,        // Current position
    line: u16,         // Current line
    line_start: usize, // Position where current line starts
    rodeo: &'a mut Rodeo,
    output: LexerOutput,
}

impl<'a> Lexer<'a> {
    pub fn new(rodeo: &'a mut Rodeo, source_code: &'a SourceCode) -> Self {
        Self {
            source_code,
            pos: 0,
            line: 0,
            line_start: 0,
            rodeo,
            output: LexerOutput::new(),
        }
    }

    /// Scans file and returns output
    pub fn scan(mut self) -> LexerOutput {
        if self.source_code.source.is_empty() {
            self.error("Empty source file", 1);
            return self.output;
        }

        while !self.is_eof() {
            let ch = self.peek();

            self.scan_token(ch)
        }

        self.push_token(TokenType::Eof, 1);

        self.output
    }

    /// Scans character and pushes token
    fn scan_token(&mut self, ch: char) {
        match ch {
            ' ' | '\t' | '\r' => {
                self.advance();
            }
            '\n' => {
                self.advance();
                self.advance_line();
            }
            '/' => {
                if self.peek_ahead() == '/' {
                    self.skip_line()
                } else {
                    self.advance();
                    self.push_token(TokenType::Operator(OperatorValue::Divide), 1);
                }
            }
            '"' => {
                let string = self.parse_string();
                let string_key = self.push_string(&string);
                self.push_token(TokenType::Literal(LiteralValue::String(string_key)), 1);
            }
            '0'..='9' => {
                let number = self.parse_number();
                match number {
                    TokenNumber::Integer(value) => {
                        self.push_token(TokenType::Literal(LiteralValue::Integer(value)), 1)
                    }
                    TokenNumber::Float(value) => {
                        self.push_token(TokenType::Literal(LiteralValue::Float(value)), 1)
                    }
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                // Parse as identifier first, then chek if it's a bool or keyword
                let name = self.parse_identifier();
                // Check for bool
                if let Some(boolean) = self.parse_boolean(name.as_str()) {
                    self.push_token(TokenType::Literal(boolean), name.len() as u16);
                }
                // Check for keyword
                else if let Some(keyword) = self.parse_keyword(name.as_str()) {
                    self.push_token(TokenType::Keyword(keyword), name.len() as u16);
                }
                // Save as id
                else {
                    let name_key = self.push_string(&name);
                    self.push_token(TokenType::Identifier(name_key), name.len() as u16);
                }
            }
            '+' | '-' | '*' | '%' | '^' | '>' | '<' | '!' | '=' | '&' | '|' => {
                let token = self.parse_operator();
                self.push_token(token, 1);
            }
            '(' | ')' | '{' | '}' | '[' | ']' | ',' | ';' | ':' => {
                let syntax = self.parse_syntax();
                self.push_token(TokenType::Syntax(syntax), 1);
            }
            _ => {
                self.advance();
                self.error("Unexpected character", 1);
            }
        }
    }

    /// Parses a string literal
    fn parse_string(&mut self) -> String {
        let mut result = String::new();
        self.advance(); // Consume opening quote

        while !self.is_eof() && self.peek() != '"' {
            if self.peek() == '\\' {
                self.advance();

                if self.is_eof() {
                    self.error("Unterminated string literal", 1);
                    break;
                }

                // Handle escape sequences
                match self.peek() {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    '0' => result.push('\0'),
                    'x' => {
                        // Hex escape sequence: \xHH
                        self.advance();

                        if self.is_eof() {
                            self.error("Incomplete hex escape sequence", 1);
                            break;
                        }

                        let hex_digit1 = self.peek();
                        self.advance();

                        if self.is_eof() {
                            self.error("Incomplete hex escape sequence", 1);
                            break;
                        }

                        let hex_digit2 = self.peek();

                        // Parse two hex digits
                        match (hex_digit1.to_digit(16), hex_digit2.to_digit(16)) {
                            (Some(d1), Some(d2)) => {
                                let byte_value = (d1 * 16 + d2) as u8;
                                result.push(byte_value as char);
                            }
                            _ => {
                                self.error(
                                    &format!(
                                        "Invalid hex escape sequence: \\x{}{}",
                                        hex_digit1, hex_digit2
                                    ),
                                    1,
                                );
                                result.push('?');
                            }
                        }
                    }
                    _ => {
                        self.error("Unknown escape sequence", 1);
                        result.push(self.peek());
                    }
                }

                self.advance();
            } else {
                result.push(self.peek());
                self.advance();
            }
        }

        if self.is_eof() {
            self.error("Unterminated string literal", 1);
        } else {
            self.advance(); // Consume closing quote
        }

        result
    }

    /// Parses a numeric literal (integer or float)
    fn parse_number(&mut self) -> TokenNumber {
        let mut number_str = String::new();
        let mut is_float = false;
        let mut has_error = false;

        // Consume all digits and at most one decimal point
        while !self.is_eof() && (self.peek().is_ascii_digit() || self.peek() == '.') {
            if self.peek() == '.' {
                if is_float {
                    self.error("Multiple decimal points in number", number_str.len() as u16);
                    has_error = true;
                } else {
                    is_float = true;
                }
            }
            number_str.push(self.peek());
            self.advance();
        }

        // Handle scientific notation (like 1e10, 1.5e-3)
        if !self.is_eof() && (self.peek() == 'e' || self.peek() == 'E') {
            is_float = true;
            number_str.push(self.peek());
            self.advance();

            // Optional sign after 'e'
            if !self.is_eof() && (self.peek() == '+' || self.peek() == '-') {
                number_str.push(self.peek());
                self.advance();
            }

            // Consume exponent digits
            while !self.is_eof() && self.peek().is_ascii_digit() {
                number_str.push(self.peek());
                self.advance();
            }
        }

        // Parse the string into a number
        if is_float {
            match number_str.parse::<f32>() {
                Ok(value) => TokenNumber::Float(value),
                Err(_) => {
                    if !has_error {
                        self.error("Invalid float literal", number_str.len() as u16);
                    }
                    TokenNumber::Float(0.0)
                }
            }
        } else {
            match number_str.parse::<i32>() {
                Ok(value) => TokenNumber::Integer(value),
                Err(_) => {
                    if !has_error {
                        self.error("Invalid integer literal", number_str.len() as u16);
                    }
                    TokenNumber::Integer(0)
                }
            }
        }
    }

    /// Parses an identifier
    fn parse_identifier(&mut self) -> String {
        let mut identifier = String::new();

        // First character is already validated (letter or underscore)
        while !self.is_eof() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            identifier.push(self.peek());
            self.advance();
        }

        identifier
    }

    /// Parses booleans
    fn parse_boolean(&mut self, string: &str) -> Option<LiteralValue> {
        match string {
            "true" => Some(LiteralValue::Boolean(true)),
            "false" => Some(LiteralValue::Boolean(false)),
            _ => None,
        }
    }

    /// Parses keywords
    fn parse_keyword(&mut self, string: &str) -> Option<KeywordValue> {
        // Check if a cast
        if self.peek() == '(' {
            return None;
        }

        match string {
            // Variables
            "let" => Some(KeywordValue::VariableDecl),
            "mut" => Some(KeywordValue::Mutable),
            "ref" => Some(KeywordValue::Reference),
            "deref" => Some(KeywordValue::Dereference),
            // Functions
            "func" => Some(KeywordValue::FunctionDecl),
            "return" => Some(KeywordValue::Return),
            // Control flow
            "if" => Some(KeywordValue::If),
            "else" => Some(KeywordValue::Else),
            "for" => Some(KeywordValue::For),
            "while" => Some(KeywordValue::While),
            // Types
            "int" => Some(KeywordValue::Integer),
            "float" => Some(KeywordValue::Float),
            "bool" => Some(KeywordValue::Boolean),
            "string" => Some(KeywordValue::String),
            "void" => Some(KeywordValue::Void),
            _ => None,
        }
    }

    /// Parses an operator token
    /// Parses both arithmetic and logic operators
    fn parse_operator(&mut self) -> TokenType {
        let ch = self.peek();
        let next = self.peek_ahead();

        let token_type = match (ch, next) {
            // Two-character operators
            ('=', '=') => TokenType::Operator(OperatorValue::Equal),
            ('!', '=') => TokenType::Operator(OperatorValue::NotEqual),
            ('<', '=') => TokenType::Operator(OperatorValue::LessEqual),
            ('>', '=') => TokenType::Operator(OperatorValue::GreaterEqual),
            ('&', '&') => TokenType::Operator(OperatorValue::And),
            ('|', '|') => TokenType::Operator(OperatorValue::Or),
            ('^', '^') => TokenType::Operator(OperatorValue::Xor),
            ('<', '<') => TokenType::Operator(OperatorValue::ShiftLeft),
            ('>', '>') => TokenType::Operator(OperatorValue::ShiftRight),

            // Single-character operators
            ('+', _) => TokenType::Operator(OperatorValue::Plus),
            ('-', _) => TokenType::Operator(OperatorValue::Minus),
            ('*', _) => TokenType::Operator(OperatorValue::Multiply),
            ('%', _) => TokenType::Operator(OperatorValue::Modulo),
            ('^', _) => TokenType::Operator(OperatorValue::Power),
            ('<', _) => TokenType::Operator(OperatorValue::Less),
            ('>', _) => TokenType::Operator(OperatorValue::Greater),
            ('!', _) => TokenType::Operator(OperatorValue::Not),

            // Assignment
            ('=', _) => TokenType::Syntax(SyntaxValue::Assign),

            // References
            ('&', _) => TokenType::Keyword(KeywordValue::Reference),

            _ => {
                self.error("Unexpected operator", 1);
                TokenType::Operator(OperatorValue::Not)
            }
        };

        // Advance based on operator length
        self.advance();
        if matches!(
            (ch, next),
            ('=', '=')
                | ('!', '=')
                | ('<', '=')
                | ('>', '=')
                | ('&', '&')
                | ('|', '|')
                | ('>', '>')
                | ('<', '<')
                | ('^', '^')
        ) {
            self.advance();
        }

        token_type
    }

    /// Parses a syntax token (punctuation and delimiters)
    fn parse_syntax(&mut self) -> SyntaxValue {
        let syntax = match self.peek() {
            '(' => SyntaxValue::LParen,
            ')' => SyntaxValue::RParen,
            '{' => SyntaxValue::LBrace,
            '}' => SyntaxValue::RBrace,
            '[' => SyntaxValue::LBracket,
            ']' => SyntaxValue::RBracket,
            ',' => SyntaxValue::Comma,
            '=' => SyntaxValue::Assign,
            ';' => SyntaxValue::Semicolon,
            ':' => SyntaxValue::Colon,
            _ => unreachable!("Invalid syntax in parse_syntax"),
        };

        self.advance();
        syntax
    }

    /// Skips everything until newline
    fn skip_line(&mut self) {
        while !self.is_eof() && self.peek() != '\n' {
            self.advance();
        }

        if !self.is_eof() {
            self.advance(); // Consume newline
            self.advance_line();
        }
    }

    /// Increments line counter
    fn advance_line(&mut self) {
        self.line += 1;
        self.line_start = self.pos - 1;
    }

    /// Consumes current token
    fn advance(&mut self) {
        self.pos += 1;
    }

    /// Peeks character without consuming it
    fn peek(&self) -> char {
        self.source_code
            .source
            .get(self.pos)
            .copied()
            .unwrap_or('\0')
    }

    /// Peeks next character without consuming it
    fn peek_ahead(&self) -> char {
        self.source_code
            .source
            .get(self.pos + 1)
            .copied()
            .unwrap_or('\0')
    }

    /// Returns true if reached end of file
    fn is_eof(&self) -> bool {
        self.pos >= self.source_code.source.len()
    }

    /// Adds token to the output
    fn push_token(&mut self, token_type: TokenType, length: u16) {
        self.output.push(Token::new(
            token_type,
            self.line,
            SourceCode::get_column(self.pos, self.line_start),
            length,
        ))
    }

    /// Pushes string to rodeo and returns key
    fn push_string(&mut self, value: &str) -> Spur {
        self.rodeo.get_or_intern(value)
    }

    /// Add error to the error list
    fn error(&mut self, message: &str, length: u16) {
        let error = CompilerError::new(
            message.to_string(),
            "Lexer".to_string(),
            self.line,
            SourceCode::get_column(self.pos, self.line_start),
            length,
            self.source_code.get_line(self.line),
            Some(self.source_code.filename.clone()),
        );

        self.output.errors.push(error);
    }
}
