use crate::interpreter::operators::OperatorType;
use crate::interpreter::tokenizer::token::Token;
use once_cell::sync::Lazy;
use regex::Regex;

static TOKENIZER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\d+\.\d+|\d+|[a-zA-Z_]+|[+\-*/()=^])").unwrap());

/// Builds a token array from raw str. Needed for AST construction.
pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    if input.trim().is_empty() {
        return Err("Input string is empty".to_string());
    }

    let cleaned = input.replace(char::is_whitespace, "").to_lowercase();
    let tokens: Vec<Token> = Vec::new();

    let tokens: Vec<Token> = TOKENIZER_REGEX
        .find_iter(&cleaned)
        .map(|m| parse_token(m.as_str(), &tokens)) // Parse tokens
        .collect::<Result<Vec<Token>, String>>()?;

    Ok(tokens)
}

fn parse_token(lexeme: &str, preceding_tokens: &[Token]) -> Result<Token, String> {
    // Try parsing as number
    if let Ok(value) = lexeme.parse::<f64>() {
        return Ok(Token::Number(value));
    }

    // Try parsing as constant
    if let Some(value) = parse_constant(lexeme) {
        return Ok(Token::Number(value));
    }

    // Parse as operator
    let mut operator = parse_operator(lexeme)?;

    // Convert to unary if needed
    if operator == OperatorType::Subtract && should_be_unary(preceding_tokens) {
        operator = OperatorType::Negate;
    }

    Ok(Token::Operator(operator))
}

fn parse_constant(lexeme: &str) -> Option<f64> {
    match lexeme {
        "pi" => Some(std::f64::consts::PI),
        "e" => Some(std::f64::consts::E),
        _ => None,
    }
}

fn parse_operator(lexeme: &str) -> Result<OperatorType, String> {
    let operator = match lexeme {
        // Arithmetic
        "+" => OperatorType::Add,
        "-" => OperatorType::Subtract,
        "*" => OperatorType::Multiply,
        "/" => OperatorType::Divide,
        // Exponents
        "^" => OperatorType::Exponent,
        "sqrt" => OperatorType::Sqrt,
        "log" => OperatorType::Log,
        "ln" => OperatorType::Ln,
        // Trigonometry
        "cos" => OperatorType::Cos,
        "sin" => OperatorType::Sin,
        "tan" => OperatorType::Tan,
        "acos" => OperatorType::Acos,
        "asin" => OperatorType::Asin,
        "atan" => OperatorType::Atan,
        // Misc
        "mod" => OperatorType::Modulo,
        "abs" => OperatorType::Abs,
        "round" => OperatorType::Round,
        // Brackets
        "(" => OperatorType::LBracket,
        ")" => OperatorType::RBracket,

        _ => return Err(format!("Unknown token: '{}'", lexeme)),
    };
    Ok(operator)
}

/// Converts binary operators to unary operators where needed.
/// A minus/plus is unary if it appears:
/// - At the start of the expression
/// - After another operator
fn should_be_unary(preceding_tokens: &[Token]) -> bool {
    // At the start of the expression
    if preceding_tokens.is_empty() {
        return true;
    }

    match &preceding_tokens[preceding_tokens.len() - 1] {
        Token::Operator(operator) => !operator.is_unary(),
        _ => false,
    }
}
