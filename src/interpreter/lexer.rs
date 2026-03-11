use crate::interpreter::tokens;
use regex::Regex;

pub fn tokenize(string: String) -> Option<Vec<tokens::Token>> {
    // Create a token vector
    let mut tokens: Vec<tokens::Token> = Vec::with_capacity(string.len());

    // Parse String into raw tokens using regex
    // This regex splits at math operators
    let regex: Regex = Regex::new(r"(\d+|[+\-*/()=])").unwrap();
    let lexeme_vec: Vec<String> = regex
        .find_iter(string.as_ref())
        .map(|m| m.as_str().to_string())
        .collect();

    // Validate string
    if string.is_empty() || string.chars().all(char::is_whitespace) {
        return None;
    }

    // Parse each token
    for lexeme in lexeme_vec {
        let token: tokens::Token = parse_token(&lexeme);
        tokens.push(token);
    }
    Some(tokens)
}

fn parse_token(lexeme: &str) -> tokens::Token {
    let mut token: tokens::Token = tokens::Token::default();

    // Handle digits
    if lexeme.parse::<f64>().is_ok() {
        let value: f64 = lexeme.parse::<f64>().unwrap();

        token.token_type = tokens::TokenType::NUMBER;
        token.value = Some(value);
    }
    // Handle symbols
    else {
        match lexeme {
            "+" => token.token_type = tokens::TokenType::ADD,
            "-" => token.token_type = tokens::TokenType::SUBTRACT,
            "*" => token.token_type = tokens::TokenType::MULTIPLY,
            "/" => token.token_type = tokens::TokenType::DIVIDE,
            _ => token.token_type = tokens::TokenType::UNKNOWN,
        }
    }
    token
}
