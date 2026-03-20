// Copyright (c) 2026 bazelik-null

use crate::morsel_interpreter::lexer::syntax_operator::SyntaxOperator;
use crate::morsel_interpreter::lexer::token::{LiteralValue, Token};

static RESERVED_KEYWORDS: &[&str] = &["let", "mut", "if", "else", "fn", "for", "while"];
static RESERVED_TYPES: &[&str] = &["float", "int", "string", "bool", "null"];

/// Tokenize input string into a token array
pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Input string is empty".to_string());
    }

    let input = strip_comments(trimmed);
    let mut tokens = Vec::with_capacity(input.len() / 5);
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '"' => {
                chars.next();
                let string = parse_string(&mut chars)?;
                tokens.push(Token::Literal(LiteralValue::String(Box::from(string))));
            }
            '0'..='9' => {
                let number = parse_number(&mut chars);
                tokens.push(number);
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let ident = parse_identifier(&mut chars);
                tokens.push(classify_identifier(&ident));
            }
            '+' | '-' | '*' | '/' | '%' | '^' | '(' | ')' | '{' | '}' | ',' | '=' | ';' | ':' => {
                chars.next();
                let op = parse_operator(ch, &tokens)?;
                tokens.push(Token::SyntaxToken(op));
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
    let mut result = String::with_capacity(input.len());

    for line in input.lines() {
        if let Some(code_part) = line.split("//").next() {
            let trimmed = code_part.trim_end();
            if !trimmed.is_empty() {
                if !result.is_empty() {
                    result.push(' ');
                }
                result.push_str(trimmed);
            }
        }
    }

    result
}

fn classify_identifier(ident: &str) -> Token {
    match ident {
        "true" => Token::Literal(LiteralValue::Boolean(true)),
        "false" => Token::Literal(LiteralValue::Boolean(false)),
        "pi" => Token::Literal(LiteralValue::Float(std::f64::consts::PI)),
        "e" => Token::Literal(LiteralValue::Float(std::f64::consts::E)),
        _ => {
            if RESERVED_KEYWORDS.contains(&ident) {
                Token::Keyword(ident.to_string())
            } else if RESERVED_TYPES.contains(&ident) {
                Token::Type(ident.to_string())
            } else {
                Token::Identifier(ident.to_string())
            }
        }
    }
}

/// Parse variable and function names
fn parse_identifier(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut ident = String::new();

    loop {
        while let Some(&ch) = chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                chars.next();
            } else {
                break;
            }
        }

        if chars.peek() == Some(&':') {
            let mut temp = chars.clone();
            temp.next();
            if temp.peek() == Some(&':') {
                chars.next();
                chars.next();
                ident.push_str("::");

                if let Some(&ch) = chars.peek()
                    && (ch.is_alphabetic() || ch == '_')
                {
                    continue;
                }
                break;
            } else {
                break;
            }
        } else {
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
        if ch.is_ascii_digit() {
            number.push(ch);
            chars.next();
        } else if ch == '.' && !is_float {
            let mut temp = chars.clone();
            temp.next();
            if temp.peek().is_some_and(|c| c.is_ascii_digit()) {
                is_float = true;
                number.push(ch);
                chars.next();
            } else {
                break;
            }
        } else {
            break;
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
