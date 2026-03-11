<div align="center">
<h1> RetardCalc </h1>
</div>

# Introduction
**RetardCalc** is my first rust project which i made just for fun. Don't expect much from it. <br>
Also, im planning to implement AST in the future!!!

# Usage
Use `interpreter::lexer::tokenize(input)` to tokenize `inpit: String` and get `Vec<Token>`. <br>
Then use `interpreter::evaluator::eval(&tokens)` to evaluate `tokens: Vec<Token>` and it will return evaluation result in `double (f64)`. <br>
Don't forget that return values are wrapped inside `Option<>`.

# Screenshot
<img width="556" height="284" alt="image" src="https://github.com/user-attachments/assets/d78ad611-2acf-4f32-b423-31cdc7c9f906" />
