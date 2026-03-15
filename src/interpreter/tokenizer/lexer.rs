use crate::interpreter::operators::OperatorType;
use crate::interpreter::tokenizer::token::Token;
use regex::Regex;

/// Builds a token array from raw str. Needed for AST construction.
pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    // Validate input
    if input.trim().is_empty() {
        return Err("Input string is empty".to_string());
    }

    // Remove whitespaces
    let cleaned = input.replace(char::is_whitespace, "");

    // Regex to capture numbers, operators and words
    let regex = Regex::new(r"(\d+\.?\d|[+\-/()=^]|[a-zA-Z_]+|\d)").unwrap();

    let mut tokens: Vec<Token> = regex
        .find_iter(&cleaned)
        .map(|m| parse_token(m.as_str())) // Parse tokens
        .collect::<Result<Vec<Token>, String>>()?;

    // Post-process to convert binary operators to unary where appropriate
    tokens = convert_unary_operators(tokens)?;

    Ok(tokens)
}

fn parse_token(lexeme: &str) -> Result<Token, String> {
    // Try parsing as number first
    if let Ok(value) = lexeme.parse::<f64>() {
        return Ok(Token::Number(value));
    }

    // Match constants
    match lexeme {
        "pi" => return Ok(Token::Number(std::f64::consts::PI)),
        "e" => return Ok(Token::Number(std::f64::consts::E)),
        "g" => return Ok(Token::Number(std::f64::consts::GOLDEN_RATIO)),
        _ => {}
    };

    // Parse as operator
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

        _ => return Err(format!("Unknown token: '{}'", lexeme)),
    };

    Ok(Token::Operator(operator))
}

/// Converts binary operators to unary operators where needed.
/// A minus/plus is unary if it appears:
/// - At the start of the expression
/// - After another operator
fn convert_unary_operators(tokens: Vec<Token>) -> Result<Vec<Token>, String> {
    let mut result = Vec::new();

    for (i, token) in tokens.into_iter().enumerate() {
        match token {
            Token::Operator(OperatorType::Subtract) => {
                // Check if this should be a unary minus
                if should_be_unary(i, &result) {
                    result.push(Token::Operator(OperatorType::Negate));
                } else {
                    result.push(token);
                }
            }
            _ => result.push(token),
        }
    }

    Ok(result)
}

/// Determines if an operator at position `index` should be treated as unary.
fn should_be_unary(index: usize, preceding_tokens: &[Token]) -> bool {
    // At the start of the expression
    if index == 0 || preceding_tokens.is_empty() {
        return true;
    }

    // Check the last token in the preceding tokens
    match &preceding_tokens[preceding_tokens.len() - 1] {
        // After another operator
        Token::Operator(op) => !matches!(op, OperatorType::Unknown),
        _ => false,
    }
}
