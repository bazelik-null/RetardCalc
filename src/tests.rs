#[cfg(test)]
use crate::cli::backend::cli_execute;

// ARITHMETIC OPERATIONS

#[test]
fn test_addition() {
    let code = "fn main() { let x = 5 + 3; }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_subtraction() {
    let code = "fn main() { let x = 10 - 4; }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_multiplication() {
    let code = "fn main() { let x = 6 * 7; }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_division() {
    let code = "fn main() { let x = 20 / 4; }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_modulo() {
    let code = "fn main() { let x = 17 % 5; }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_negation() {
    let code = "fn main() { let x = -42; }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_complex_arithmetic_expression() {
    let code = "fn main() { let x = 10 + 5 * 2 - 3; }";
    assert!(cli_execute(code, false).is_ok());
}

// EXPONENTS AND ROOTS

#[test]
fn test_exponentiation() {
    let code = "fn main() { let x = 2 ^ 8; }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_square_root() {
    let code = "fn main() { let x = std::math::sqrt(16); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_cubic_root() {
    let code = "fn main() { let x = std::math::cbrt(27); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_nth_root() {
    let code = "fn main() { let x = std::math::root(16, 4); }";
    assert!(cli_execute(code, false).is_ok());
}

// LOGARITHMS

#[test]
fn test_logarithm() {
    let code = "fn main() { let x = std::math::log(10, 100); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_natural_logarithm() {
    let code = "fn main() { let x = std::math::ln(2.718); }";
    assert!(cli_execute(code, false).is_ok());
}

// TRIGONOMETRIC FUNCTIONS

#[test]
fn test_cosine() {
    let code = "fn main() { let x = std::math::cos(0); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_sine() {
    let code = "fn main() { let x = std::math::sin(0); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_tangent() {
    let code = "fn main() { let x = std::math::tan(0); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_arccosine() {
    let code = "fn main() { let x = std::math::acos(1); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_arcsine() {
    let code = "fn main() { let x = std::math::asin(0); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_arctangent() {
    let code = "fn main() { let x = std::math::atan(1); }";
    assert!(cli_execute(code, false).is_ok());
}

// UTILITY FUNCTIONS

#[test]
fn test_absolute_value() {
    let code = "fn main() { let x = std::math::abs(-42); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_rounding() {
    let code = "fn main() { let x = std::math::round(3.7); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_floor() {
    let code = "fn main() { let x = std::math::floor(3.9); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_ceiling() {
    let code = "fn main() { let x = std::math::ceil(3.1); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_max_two_values() {
    let code = "fn main() { let x = std::math::max(10, 20); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_max_multiple_values() {
    let code = "fn main() { let x = std::math::max(5, 15, 10, 20, 3); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_min_two_values() {
    let code = "fn main() { let x = std::math::min(10, 20); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_min_multiple_values() {
    let code = "fn main() { let x = std::math::min(5, 15, 10, 20, 3); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_println_single_value() {
    let code = "fn main() { std::io::println(42); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_println_multiple_values() {
    let code = "fn main() { std::io::println(10, 20, \"hello\", true); }";
    assert!(cli_execute(code, false).is_ok());
}

// TYPE INFERENCE WITH OPERATIONS

#[test]
fn test_float_arithmetic() {
    let code = "fn main() { let x = 3.5 + 2.5; }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_mixed_int_float_arithmetic() {
    let code = "fn main() { let x = 10 + 3.5; }";
    assert!(cli_execute(code, false).is_ok());
}

// VARIABLE OPERATIONS WITH FUNCTIONS

#[test]
fn test_variable_with_sqrt_function() {
    let code = "fn main() { let x = std::math::sqrt(25); }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_variable_reassignment_with_arithmetic() {
    let code = "fn main() { let mut x = 10; x = 20 + 5; }";
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_function_with_arithmetic_arguments() {
    let code = r#"
        fn main() {
            fn calculate(a: int, b: int) {
                a + b
            }
            calculate(5, 10);
        }
    "#;
    assert!(cli_execute(code, false).is_ok());
}

#[test]
fn test_nested_function_calls_with_math() {
    let code = r#"
        fn main() {
            fn double(x: int) {
                let result: int = x * 2;
                result;
            }
            let val: int = double(5);
        }
    "#;
    assert!(cli_execute(code, false).is_ok());
}

// EDGE CASES AND ERROR HANDLING

#[test]
fn test_division_by_zero_fails() {
    let code = "fn main() { let x = 10 / 0; }";
    assert!(cli_execute(code, false).is_err());
}

#[test]
fn test_invalid_function_call_fails() {
    let code = "fn main() { let x = invalid_func(5); }";
    assert!(cli_execute(code, false).is_err());
}

#[test]
fn test_invalid_function_namespace_call_fails() {
    let code = "fn main() { let x = std::io::cos(5); }";
    assert!(cli_execute(code, false).is_err());
}

#[test]
fn test_recursion() {
    let code = "fn main() { main() }";
    assert!(cli_execute(code, false).is_err());
}

#[test]
fn test_wrong_number_of_arguments_fails() {
    let code = "fn main() { let x = cos(5, 6); }";
    assert!(cli_execute(code, false).is_err());
}

#[test]
fn test_type_mismatch_in_function_fails() {
    let code = r#"
        fn main() {
            fn add(a: int, b: int) {}
            add(5, "string");
        }
    "#;
    assert!(cli_execute(code, false).is_err());
}

#[test]
fn test_reassign_immutable_variable_fails() {
    let code = "fn main() { let x = 10; x = 20; }";
    assert!(cli_execute(code, false).is_err());
}
