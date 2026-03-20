// Copyright (c) 2026 bazelik-null

use crate::morsel_interpreter::lexer::syntax_operator::SyntaxOperator;
use crate::morsel_interpreter::lexer::token::{LiteralValue, Token};

static RESERVED_KEYWORDS: &[&str] = &["let", "mut", "if", "else", "fn", "for", "while"];
static RESERVED_TYPES: &[&str] = &["float", "int", "string", "bool", "null"];

/// Tokenize input string into a token array
pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    if input.trim().is_empty() {
        return Err("Input string is empty".to_string());
    }

    let input = strip_comments(input);
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            // Skip whitespace
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            // String literals
            '"' => {
                chars.next();
                let string = parse_string(&mut chars)?;
                tokens.push(Token::Literal(LiteralValue::String(string)));
            }
            // Numbers
            '0'..='9' => {
                let number = parse_number(&mut chars);
                tokens.push(number);
            }
            // Identifiers, keywords, functions, types
            'a'..='z' | 'A'..='Z' | '_' => {
                let ident = parse_identifier(&mut chars);
                tokens.push(classify_identifier(&ident));
            }
            // Operators and punctuation
            '+' | '-' | '*' | '/' | '%' | '^' | '(' | ')' | '{' | '}' | ',' | '=' | ';' => {
                chars.next();
                let op = parse_operator(ch, &tokens)?;
                tokens.push(Token::SyntaxToken(op));
            }
            ':' => {
                // Check if this is part of a namespace (::)
                if chars.clone().nth(1) == Some(':') {
                    // This will be handled as part of identifier parsing
                    return Err("Unexpected '::' outside of identifier context".to_string());
                } else {
                    chars.next();
                    let op = parse_operator(ch, &tokens)?;
                    tokens.push(Token::SyntaxToken(op));
                }
            }
            _ => {
                return Err(format!("Unexpected character: '{}'", ch));
            }
        }
    }

    Ok(tokens)
}

/// Remove comments from input (// to end of line)
fn strip_comments(input: &str) -> String {
    input
        .lines()
        .filter_map(|line| {
            let trimmed = line.split("//").next()?.trim_end();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn classify_identifier(ident: &str) -> Token {
    if let Some(value) = parse_boolean(ident) {
        return Token::Literal(LiteralValue::Boolean(value));
    }

    if let Some(value) = parse_constant(ident) {
        return Token::Literal(LiteralValue::Float(value));
    }

    if RESERVED_KEYWORDS.contains(&ident) {
        return Token::Keyword(ident.to_string());
    }

    if RESERVED_TYPES.contains(&ident) {
        return Token::Type(ident.to_string());
    }

    Token::Identifier(ident.to_string())
}

/// Parse constants
fn parse_constant(lexeme: &str) -> Option<f64> {
    match lexeme {
        "pi" => Some(std::f64::consts::PI),
        "e" => Some(std::f64::consts::E),
        _ => None,
    }
}

/// Parse boolean literals
fn parse_boolean(lexeme: &str) -> Option<bool> {
    match lexeme {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Parse variable names with namespace support (e.g., helpers::tools::add)
fn parse_identifier(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut ident = String::new();

    loop {
        // Parse identifier segment (alphanumeric + underscore)
        while let Some(&ch) = chars.peek() {
            match ch {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                    ident.push(ch);
                    chars.next();
                }
                _ => break,
            }
        }

        // Check for namespace separator (::)
        if chars.clone().next() == Some(':') && chars.clone().nth(1) == Some(':') {
            // Consume the :: and continue parsing the next segment
            chars.next(); // consume first :
            chars.next(); // consume second :
            ident.push_str("::");

            // Ensure there's a valid identifier after ::
            if let Some(&ch) = chars.peek() {
                match ch {
                    'a'..='z' | 'A'..='Z' | '_' => {
                        // Valid start of next segment, continue loop
                        continue;
                    }
                    _ => {
                        // Invalid character after ::, break and let parser handle error
                        break;
                    }
                }
            } else {
                // End of input after ::, break
                break;
            }
        } else {
            // No namespace separator, we're done
            break;
        }
    }

    ident
}

/// Parse string literals (removes quotes and handles escape sequences)
fn parse_string(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String, String> {
    let mut result = String::new();
    let mut escaped = false;

    while let Some(&ch) = chars.peek() {
        chars.next();

        if escaped {
            match ch {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                'r' => result.push('\r'),
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                _ => {
                    result.push('\\');
                    result.push(ch);
                }
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Ok(result);
        } else {
            result.push(ch);
        }
    }

    Err("Unterminated string literal".to_string())
}

/// Parse numbers
fn parse_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Token {
    let mut number = String::new();
    let mut is_float = false;

    while let Some(&ch) = chars.peek() {
        match ch {
            '0'..='9' => {
                number.push(ch);
                chars.next();
            }
            '.' if !is_float && chars.clone().nth(1).is_some_and(|c| c.is_ascii_digit()) => {
                is_float = true;
                number.push(ch);
                chars.next();
            }
            _ => break,
        }
    }

    if is_float {
        Token::Literal(LiteralValue::Float(number.parse().unwrap()))
    } else {
        Token::Literal(LiteralValue::Integer(number.parse().unwrap()))
    }
}

/// Parse operators
fn parse_operator(ch: char, preceding_tokens: &[Token]) -> Result<SyntaxOperator, String> {
    let op = match ch {
        '+' => SyntaxOperator::Add,
        '-' => SyntaxOperator::Subtract,
        '*' => SyntaxOperator::Multiply,
        '/' => SyntaxOperator::Divide,
        '%' => SyntaxOperator::Modulo,
        '^' => SyntaxOperator::Exponent,
        '(' => SyntaxOperator::LParen,
        ')' => SyntaxOperator::RParen,
        '{' => SyntaxOperator::CurlyLParen,
        '}' => SyntaxOperator::CurlyRParen,
        ',' => SyntaxOperator::Comma,
        '=' => SyntaxOperator::Assign,
        ';' => SyntaxOperator::Semicolon,
        ':' => SyntaxOperator::Colon,
        _ => return Err(format!("Unknown operator: {}", ch)),
    };

    let final_op = if op == SyntaxOperator::Subtract && should_be_unary(preceding_tokens) {
        SyntaxOperator::Negate
    } else {
        op
    };

    Ok(final_op)
}

/// Determine if a minus operator should be treated as unary negation
fn should_be_unary(preceding_tokens: &[Token]) -> bool {
    match preceding_tokens.last() {
        None => true,
        Some(Token::SyntaxToken(op)) => !matches!(op, SyntaxOperator::RParen),
        Some(Token::Keyword(_)) => true,
        Some(Token::Literal(_) | Token::Identifier(_) | Token::Type(_)) => false,
    }
}
