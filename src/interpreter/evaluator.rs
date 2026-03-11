use crate::interpreter::tokens;

pub fn eval(tokens: &Vec<tokens::Token>) -> Result<f64, &'static str> {
    if tokens.is_empty() {
        return Err("No tokens found");
    }

    let mut result: f64 = 0.0;

    // Track the current operation
    let mut operation: tokens::TokenType = tokens::TokenType::ADD;

    // Init previous tokens::Token register at the first tokens::Token
    let mut previous_token: &tokens::Token = tokens.first().unwrap();

    for token in tokens {
        match token.token_type {
            tokens::TokenType::ADD => {
                result = apply_operation(result, previous_token.value.unwrap_or(0.0), operation);
                operation = tokens::TokenType::ADD;
            }
            tokens::TokenType::SUBTRACT => {
                result = apply_operation(result, previous_token.value.unwrap_or(0.0), operation);
                operation = tokens::TokenType::SUBTRACT;
            }
            tokens::TokenType::MULTIPLY => {
                result = apply_operation(result, previous_token.value.unwrap_or(0.0), operation);
                operation = tokens::TokenType::MULTIPLY;
            }
            tokens::TokenType::DIVIDE => {
                result = apply_operation(result, previous_token.value.unwrap_or(0.0), operation);
                operation = tokens::TokenType::DIVIDE;
            }
            _ => {}
        }

        previous_token = token;
    }

    // Apply the last operation to the last number
    result = apply_operation(result, previous_token.value.unwrap_or(0.0), operation);
    Ok(result)
}

fn apply_operation(result: f64, value: f64, operation: tokens::TokenType) -> f64 {
    match operation {
        tokens::TokenType::ADD => result + value,
        tokens::TokenType::SUBTRACT => result - value,
        tokens::TokenType::MULTIPLY => result * value,
        tokens::TokenType::DIVIDE => result / value,
        _ => result,
    }
}
