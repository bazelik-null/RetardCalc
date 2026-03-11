<div align="center">
<h1> RetardCalc </h1>
</div>

# Introduction
**RetardCalc** is my first rust project which I made just for fun. Don't expect much from it. <br>
Also, im planning to implement AST in the future!!!

# Usage
Example code
```rust
// Takes raw input string, parses it and evaluates it.
// Returns either calculated value, or error.
fn calculate(input: String) -> Result<f64, Box<dyn Error>> {
    // Parse expression
    let tokens: Vec<tokens::Token> = interpreter::lexer::tokenize(&input)?;

    // Evaluate expression
    let eval: f64 = interpreter::evaluator::eval(&tokens)?;

    Ok(eval)
}
```

# Screenshot
<img width="556" height="284" alt="image" src="https://github.com/user-attachments/assets/d78ad611-2acf-4f32-b423-31cdc7c9f906" />
