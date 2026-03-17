<div align="center">
<h1> Morsel </h1>
<img src="doc/img/logo.png" width="500">
</div>

> [!WARNING]
>
>Work in progress.

# Introduction

**Morsel** is an interpreted programming language built in Rust as my first Rust project. <br>
**Morsel** evaluates expressions using a three-stage pipeline:

1. **Lexer** (`lexer::tokenize`) - Converts input string into tokens
2. **Parser** (`parser::parse`) - Builds an Abstract Syntax Tree (AST) from tokens
3. **Evaluator** (`evaluator::eval`) - Traverses the AST and computes the result

# Available Functions

### Arithmetic Operations

- **Addition:** `x + y`
- **Subtraction:** `x - y`
- **Multiplication:** `x * y`
- **Division:** `x / y`

### Exponent and Logarithmic Operations

- **Exponentiation:** `x ^ y`
- **Square root:** `sqrt(x)`
- **Cubic root:** `cbrt(x)`
- **Root:** `root(x, y)` (where x is radicant, y is degree)
- **Logarithm:** `log(x, y)` (where x is base, y is argument)
- **Natural logarithm:** `ln(x)`

### Trigonometric Functions

- **Cosine:** `cos(x)`
- **Sine:** `sin(x)`
- **Tangent:** `tan(x)`
- **Arccosine:** `acos(x)`
- **Arcsine:** `asin(x)`
- **Arctangent:** `atan(x)`

### Miscellaneous Operations

- **Negation:** `-x`
- **Modulo (remainder):** `x % y`
- **Absolute value:** `abs(x)`
- **Rounding:** `round(x)`
- **Max value:** `max(x, ...)`
- **Min value:** `min(x, ...)`
- **Floor:** `floor(x)`
- **Ceil:** `ceil(x)`

### Tools

- **Printing:** `print(x)`

# Available keywords

- **Define a variable:** `let x = y;` (definition requires initialization)

# Available features

- **Assign a variable:** `x = y;`
- **Comments:** `// Comment`

# Project Structure

1. **Entry point** (`src/main.rs`) - Launches CLI or evaluates file from argument, outputting only the result.
2. **Command Line Interface** (`src/cli/`) - User interface that accepts commands and file inputs.
3. **Core** (`src/morsel_core/`) - Core of the Morsel interpreter.
    1. **Interpreter** (`src/morsel_core/interpreter.rs`) - Wrapper for easy code execution.
    2. **Lexer** (`src/morsel_core/lexing/`) - Tokenizes input string into an array of tokens.
    3. **Parser** (`src/morsel_core/parsing/`) - Builds an Abstract Syntax Tree from tokens.
    4. **Evaluator** (`src/morsel_core/evaluating/`) - Evaluates the AST and returns the result.

# Screenshot

<img src="doc/img/screenshot.png">
