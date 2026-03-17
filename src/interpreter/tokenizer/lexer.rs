use crate::interpreter::operators::OperatorType;
use crate::interpreter::tokenizer::token::Token;
use once_cell::sync::Lazy;
use regex::Regex;

static TOKENIZER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\d+\.\d+|\d+|[a-zA-Z_]+|[+\-*/()=^%])").unwrap());

/// Builds a token array from raw input string
pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    if input.trim().is_empty() {
        return Err("Input string is empty".to_string());
    }

    let cleaned = input.replace(char::is_whitespace, "").to_lowercase();
    let mut tokens = Vec::new();

    for m in TOKENIZER_REGEX.find_iter(&cleaned) {
        let token = parse_token(m.as_str(), &tokens)?;
        tokens.push(token);
    }

    Ok(tokens)
}

fn parse_token(lexeme: &str, preceding_tokens: &[Token]) -> Result<Token, String> {
    // Try parsing as number
    if let Ok(value) = lexeme.parse::<f64>() {
        return Ok(Token::Number(value));
    }

    // Try parsing as constant
    if let Some(value) = try_parse_constant(lexeme) {
        return Ok(Token::Number(value));
    }

    // Parse as operator
    let mut op = parse_operator(lexeme)?;

    // Convert subtract to negate if it's a unary operator
    if op == OperatorType::Subtract && is_unary_position(preceding_tokens) {
        op = OperatorType::Negate;
    }

    Ok(Token::Operator(op))
}

/// Attempts to parse a constant like pi or e
fn try_parse_constant(lexeme: &str) -> Option<f64> {
    Some(match lexeme {
        "pi" => std::f64::consts::PI,
        "e" => std::f64::consts::E,
        _ => return None,
    })
}

/// Maps lexeme strings to operator types
fn parse_operator(lexeme: &str) -> Result<OperatorType, String> {
    let op = match lexeme {
        // Arithmetic
        "+" => OperatorType::Add,
        "-" => OperatorType::Subtract,
        "*" => OperatorType::Multiply,
        "/" => OperatorType::Divide,
        "%" => OperatorType::Modulo,

        // Exponentiation
        "^" => OperatorType::Exponent,
        "sqrt" => OperatorType::Sqrt,
        "ln" => OperatorType::Ln,
        "log" => OperatorType::Log,

        // Trigonometry
        "sin" => OperatorType::Sin,
        "cos" => OperatorType::Cos,
        "tan" => OperatorType::Tan,
        "asin" => OperatorType::Asin,
        "acos" => OperatorType::Acos,
        "atan" => OperatorType::Atan,

        // Misc
        "abs" => OperatorType::Abs,
        "round" => OperatorType::Round,

        // Brackets
        "(" => OperatorType::LParen,
        ")" => OperatorType::RParen,

        _ => return Err(format!("Unknown token: '{}'", lexeme)),
    };

    Ok(op)
}

/// Determines if a minus operator should be treated as unary negation
/// This happens when it appears at the start or after another operator
fn is_unary_position(preceding_tokens: &[Token]) -> bool {
    match preceding_tokens.last() {
        None => true,                     // Start of expression
        Some(Token::Operator(_)) => true, // After any operator
        Some(Token::Number(_)) => false,  // After a number (binary minus)
    }
}
