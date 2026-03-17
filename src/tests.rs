// Copyright (c) 2026 bazelik-null

#[cfg(test)]
use crate::cli::backend::calculate_with_result;

#[allow(dead_code)]
fn assert_approx_eq(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() < tolerance,
        "expected {}, got {}",
        expected,
        actual
    );
}

// ARITHMETIC OPERATORS

#[test]
fn test_addition() {
    assert_eq!(calculate_with_result("2 + 3", false), Ok(5.0));
    assert_eq!(calculate_with_result("0 + 0", false), Ok(0.0));
    assert_eq!(calculate_with_result("-5 + 3", false), Ok(-2.0));
    assert_eq!(calculate_with_result("100 + 50", false), Ok(150.0));
}

#[test]
fn test_subtraction() {
    assert_eq!(calculate_with_result("10 - 4", false), Ok(6.0));
    assert_eq!(calculate_with_result("5 - 5", false), Ok(0.0));
    assert_eq!(calculate_with_result("3 - 10", false), Ok(-7.0));
    assert_eq!(calculate_with_result("-5 - 3", false), Ok(-8.0));
}

#[test]
fn test_multiplication() {
    assert_eq!(calculate_with_result("3 * 7", false), Ok(21.0));
    assert_eq!(calculate_with_result("0 * 100", false), Ok(0.0));
    assert_eq!(calculate_with_result("-2 * 5", false), Ok(-10.0));
    assert_eq!(calculate_with_result("2.5 * 4", false), Ok(10.0));
}

#[test]
fn test_division() {
    assert_eq!(calculate_with_result("20 / 4", false), Ok(5.0));
    assert_eq!(calculate_with_result("10 / 2", false), Ok(5.0));
    assert_eq!(calculate_with_result("7 / 2", false), Ok(3.5));
    assert_eq!(calculate_with_result("-10 / 2", false), Ok(-5.0));
}

#[test]
fn test_division_by_zero() {
    assert!(calculate_with_result("5 / 0", false).unwrap().is_infinite());
}

#[test]
fn test_modulo() {
    assert_eq!(calculate_with_result("10 % 3", false), Ok(1.0));
    assert_eq!(calculate_with_result("20 % 5", false), Ok(0.0));
    assert_eq!(calculate_with_result("7 % 2", false), Ok(1.0));
    assert_eq!(calculate_with_result("-10 % 3", false), Ok(2.0));
}

// EXPONENTS & LOGARITHMS

#[test]
fn test_exponent() {
    assert_eq!(calculate_with_result("2 ^ 3", false), Ok(8.0));
    assert_eq!(calculate_with_result("5 ^ 2", false), Ok(25.0));
    assert_eq!(calculate_with_result("10 ^ 0", false), Ok(1.0));
    assert_eq!(calculate_with_result("2 ^ -1", false), Ok(0.5));
}

#[test]
fn test_sqrt() {
    assert_eq!(calculate_with_result("sqrt(4)", false), Ok(2.0));
    assert_eq!(calculate_with_result("sqrt(9)", false), Ok(3.0));
    assert_eq!(calculate_with_result("sqrt(16)", false), Ok(4.0));
    assert_eq!(calculate_with_result("sqrt(0.25)", false), Ok(0.5));
}

#[test]
fn test_sqrt_negative() {
    assert!(calculate_with_result("sqrt(-1)", false).unwrap().is_nan());
}

#[test]
fn test_natural_logarithm() {
    assert_eq!(calculate_with_result("ln(1)", false), Ok(0.0));
    let result = calculate_with_result("ln(e)", false).unwrap();
    assert_approx_eq(result, 1.0, 0.0001);
}

#[test]
fn test_logarithm() {
    let result = calculate_with_result("log(10, 100)", false).unwrap();
    assert_approx_eq(result, 2.0, 0.0001);

    let result = calculate_with_result("log(2, 8)", false).unwrap();
    assert_approx_eq(result, 3.0, 0.0001);
}

#[test]
fn test_cosine() {
    assert_eq!(calculate_with_result("cos(0)", false), Ok(1.0));
    let result = calculate_with_result("cos(pi)", false).unwrap();
    assert_approx_eq(result, -1.0, 0.0001);
}

#[test]
fn test_sine() {
    assert_eq!(calculate_with_result("sin(0)", false), Ok(0.0));
    let result = calculate_with_result("sin(pi / 2)", false).unwrap();
    assert_approx_eq(result, 1.0, 0.0001);
}

#[test]
fn test_arccosine() {
    assert_eq!(calculate_with_result("acos(1)", false), Ok(0.0));
    let result = calculate_with_result("acos(0)", false).unwrap();
    assert_approx_eq(result, std::f64::consts::PI / 2.0, 0.0001);
}

#[test]
fn test_arcsine() {
    assert_eq!(calculate_with_result("asin(0)", false), Ok(0.0));
    let result = calculate_with_result("asin(1)", false).unwrap();
    assert_approx_eq(result, std::f64::consts::PI / 2.0, 0.0001);
}

#[test]
fn test_arctangent() {
    assert_eq!(calculate_with_result("atan(0)", false), Ok(0.0));
    let result = calculate_with_result("atan(1)", false).unwrap();
    assert_approx_eq(result, std::f64::consts::PI / 4.0, 0.0001);
}

// MISCELLANEOUS FUNCTIONS

#[test]
fn test_negate() {
    assert_eq!(calculate_with_result("-5", false), Ok(-5.0));
    assert_eq!(calculate_with_result("-(-10)", false), Ok(10.0));
    assert_eq!(calculate_with_result("-(2 + 3)", false), Ok(-5.0));
}

#[test]
fn test_absolute_value() {
    assert_eq!(calculate_with_result("abs(5)", false), Ok(5.0));
    assert_eq!(calculate_with_result("abs(-10)", false), Ok(10.0));
    assert_eq!(calculate_with_result("abs(-3.5)", false), Ok(3.5));
    assert_eq!(calculate_with_result("abs(0)", false), Ok(0.0));
}

#[test]
fn test_round() {
    assert_eq!(calculate_with_result("round(3.14)", false), Ok(3.0));
    assert_eq!(calculate_with_result("round(3.5)", false), Ok(4.0));
    assert_eq!(calculate_with_result("round(3.6)", false), Ok(4.0));
    assert_eq!(calculate_with_result("round(-2.5)", false), Ok(-3.0));
}

// CONSTANTS

#[test]
fn test_pi_constant() {
    let result = calculate_with_result("pi", false).unwrap();
    assert_approx_eq(result, std::f64::consts::PI, 0.0001);
}

#[test]
fn test_e_constant() {
    let result = calculate_with_result("e", false).unwrap();
    assert_approx_eq(result, std::f64::consts::E, 0.0001);
}

#[test]
fn test_pi_in_expression() {
    let result = calculate_with_result("2 * pi", false).unwrap();
    assert_approx_eq(result, 2.0 * std::f64::consts::PI, 0.0001);
}

#[test]
fn test_e_in_expression() {
    let result = calculate_with_result("e ^ 2", false).unwrap();
    assert_approx_eq(result, std::f64::consts::E.powi(2), 0.0001);
}

// OPERATOR PRECEDENCE

#[test]
fn test_precedence_multiplication_before_addition() {
    assert_eq!(calculate_with_result("2 + 3 * 4", false), Ok(14.0));
}

#[test]
fn test_precedence_division_before_subtraction() {
    assert_eq!(calculate_with_result("20 - 10 / 2", false), Ok(15.0));
}

#[test]
fn test_precedence_exponent_before_multiplication() {
    assert_eq!(calculate_with_result("2 * 3 ^ 2", false), Ok(18.0));
}

// PARENTHESES

#[test]
fn test_parentheses_simple() {
    assert_eq!(calculate_with_result("(2 + 3) * 4", false), Ok(20.0));
}

#[test]
fn test_parentheses_nested() {
    assert_eq!(calculate_with_result("((2 + 3) * 4) / 2", false), Ok(10.0));
}

#[test]
fn test_parentheses_with_functions() {
    assert_eq!(
        calculate_with_result("(sqrt(4) + sqrt(9)) * 2", false),
        Ok(10.0)
    );
}

#[test]
fn test_unmatched_parentheses() {
    assert!(calculate_with_result("(2 + 3", false).is_err());
    assert!(calculate_with_result("2 + 3)", false).is_err());
}

// COMPLEX EXPRESSIONS

#[test]
fn test_complex_expressions() {
    assert_eq!(calculate_with_result("(2 + 3) * (4 - 1)", false), Ok(15.0));
    assert_eq!(
        calculate_with_result("sqrt(16) + abs(-5) * 2", false),
        Ok(14.0)
    );
    assert_eq!(calculate_with_result("2 ^ 3 + 3 ^ 2", false), Ok(17.0));
    assert_eq!(calculate_with_result("sin(0) + cos(0)", false), Ok(1.0));
}

// VARIABLES

#[test]
fn test_variables() {
    assert_eq!(
        calculate_with_result("let x = 1; let y = 2; x = x + y; x", false),
        Ok(3.0)
    );
    assert_eq!(
        calculate_with_result("let y = 2; let x = y * 5; x", false),
        Ok(10.0)
    );
    assert_approx_eq(
        calculate_with_result("let y = pi; let x = sin(y); x", false).unwrap(),
        0.0,
        0.0001,
    );
    assert_eq!(
        calculate_with_result("let y = 4; let x = sqrt(y) * 2^2; x", false),
        Ok(8.0)
    );
}

// EDGE CASES

#[test]
fn test_edge_cases() {
    assert_eq!(calculate_with_result("0", false), Ok(0.0));
    assert_eq!(calculate_with_result("-42", false), Ok(-42.0));
    assert_eq!(calculate_with_result("0.5 + 0.5", false), Ok(1.0));
    assert_eq!(
        calculate_with_result("1000000 + 1000000", false),
        Ok(2000000.0)
    );
    assert_eq!(calculate_with_result("0.0001 + 0.0001", false), Ok(0.0002));
}

// ERROR CASES

#[test]
fn test_error_cases() {
    assert!(calculate_with_result("2 +", false).is_err());
    assert!(calculate_with_result("", false).is_err());
    assert!(calculate_with_result("   ", false).is_err());
    assert!(calculate_with_result("2 + + 3", false).is_err());
    assert!(calculate_with_result("unknown(5)", false).is_err());
}
