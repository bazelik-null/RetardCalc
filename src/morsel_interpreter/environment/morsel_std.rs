// Copyright (c) 2026 bazelik-null

use crate::morsel_interpreter::environment::symbol_table::function_symbol::{
    FunctionParamSymbol, FunctionSymbol, FunctionSymbolTable,
};
use crate::morsel_interpreter::environment::types::Type;
use crate::morsel_interpreter::environment::value::Value;
use std::io;
use std::io::Write;

const BUILTIN_FUNCTIONS: &[(&str, &str, usize, Type, Type, bool)] = &[
    // Math - Trigonometric
    ("sin", "std::math", 1, Type::Float, Type::Float, false),
    ("cos", "std::math", 1, Type::Float, Type::Float, false),
    ("tan", "std::math", 1, Type::Float, Type::Float, false),
    ("asin", "std::math", 1, Type::Float, Type::Float, false),
    ("acos", "std::math", 1, Type::Float, Type::Float, false),
    ("atan", "std::math", 1, Type::Float, Type::Float, false),
    // Math - Roots
    ("sqrt", "std::math", 1, Type::Float, Type::Float, false),
    ("cbrt", "std::math", 1, Type::Float, Type::Float, false),
    ("ln", "std::math", 1, Type::Float, Type::Float, false),
    // Math - Rounding
    ("round", "std::math", 1, Type::Float, Type::Float, false),
    ("floor", "std::math", 1, Type::Float, Type::Float, false),
    ("ceil", "std::math", 1, Type::Float, Type::Float, false),
    // Math - Misc
    ("abs", "std::math", 1, Type::Float, Type::Float, false),
    // Math - Multi-arg
    ("root", "std::math", 2, Type::Float, Type::Float, false),
    ("log", "std::math", 2, Type::Float, Type::Float, false),
    ("max", "std::math", 1, Type::Float, Type::Float, true), // Min 1 arg
    ("min", "std::math", 1, Type::Float, Type::Float, true), // Min 1 arg
    // I/O
    ("println", "std::io", 0, Type::Null, Type::Any, true), // 0+ args
    ("print", "std::io", 0, Type::Null, Type::Any, true),   // 0+ args
    ("input", "std::io", 1, Type::String, Type::String, false), // 1 arg (prompt)
    // Type Casting
    ("to_int", "std", 1, Type::Integer, Type::Any, false),
    ("to_float", "std", 1, Type::Float, Type::Any, false),
    ("to_bool", "std", 1, Type::Boolean, Type::Any, false),
    ("to_string", "std", 1, Type::String, Type::Any, false),
];

/// Dispatcher for builtin function implementations
pub struct BuiltinFunctionDispatcher;

impl BuiltinFunctionDispatcher {
    /// Call a builtin function by name with arguments
    pub fn call(name: &str, namespace: &str, args: &[Value]) -> Result<Value, String> {
        match namespace {
            // Functions grouped by namespace.
            "std::math" => Self::call_math_function(name, args),
            "std::io" => Self::call_io_function(name, args),
            "std" => Self::call_type_conversion_function(name, args),
            _ => Err(format!("Unknown builtin function: '{}'", name)),
        }
    }

    fn call_math_function(name: &str, args: &[Value]) -> Result<Value, String> {
        match name {
            "sin" => Self::sin(args),
            "cos" => Self::cos(args),
            "tan" => Self::tan(args),
            "asin" => Self::asin(args),
            "acos" => Self::acos(args),
            "atan" => Self::atan(args),
            "sqrt" => Self::sqrt(args),
            "cbrt" => Self::cbrt(args),
            "ln" => Self::ln(args),
            "root" => Self::root(args),
            "log" => Self::log(args),
            "round" => Self::round(args),
            "floor" => Self::floor(args),
            "ceil" => Self::ceil(args),
            "abs" => Self::abs(args),
            "max" => Self::max(args),
            "min" => Self::min(args),
            _ => Err(format!("Unknown math function: '{}'", name)),
        }
    }

    fn call_io_function(name: &str, args: &[Value]) -> Result<Value, String> {
        match name {
            "print" => Self::print(args),
            "println" => Self::println(args),
            "input" => Self::input(args),
            _ => Err(format!("Unknown I/O function: '{}'", name)),
        }
    }

    fn call_type_conversion_function(name: &str, args: &[Value]) -> Result<Value, String> {
        match name {
            "to_int" => Self::cast_to_int(args),
            "to_float" => Self::cast_to_float(args),
            "to_bool" => Self::cast_to_bool(args),
            "to_string" => Self::cast_to_string(args),
            _ => Err(format!("Unknown type conversion function: '{}'", name)),
        }
    }

    pub fn register_builtins(symbols: &mut FunctionSymbolTable) {
        for &(name, namespace, param_count, return_type, param_type, is_variadic) in
            BUILTIN_FUNCTIONS
        {
            let params = (0..param_count)
                .map(|i| FunctionParamSymbol::new(format!("arg{}", i), param_type))
                .collect();

            let mut func = FunctionSymbol::builtin(
                name.to_string(),
                namespace.to_string(),
                params,
                return_type,
            );
            func.is_variadic = is_variadic;

            let _ = symbols.define(func);
        }
    }

    // ---- Math Functions Implementation ----
    fn sin(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Float(args[0].to_float()?.sin()))
    }
    fn cos(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Float(args[0].to_float()?.cos()))
    }
    fn tan(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Float(args[0].to_float()?.tan()))
    }
    fn asin(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Float(args[0].to_float()?.asin()))
    }
    fn acos(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Float(args[0].to_float()?.acos()))
    }
    fn atan(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Float(args[0].to_float()?.atan()))
    }
    fn sqrt(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Float(args[0].to_float()?.sqrt()))
    }
    fn cbrt(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Float(args[0].to_float()?.cbrt()))
    }
    fn ln(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Float(args[0].to_float()?.ln()))
    }
    fn root(args: &[Value]) -> Result<Value, String> {
        let base = args[0].to_float()?;
        let degree = args[1].to_float()?;
        Ok(Value::Float(base.powf(1.0 / degree)))
    }
    fn log(args: &[Value]) -> Result<Value, String> {
        let base = args[0].to_float()?;
        let value = args[1].to_float()?;
        Ok(Value::Float(value.log(base)))
    }
    fn round(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Float(args[0].to_float()?.round()))
    }
    fn floor(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Float(args[0].to_float()?.floor()))
    }
    fn ceil(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Float(args[0].to_float()?.ceil()))
    }
    fn abs(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Float(args[0].to_float()?.abs()))
    }
    fn max(args: &[Value]) -> Result<Value, String> {
        args.iter()
            .try_fold(f64::NEG_INFINITY, |acc, v| v.to_float().map(|f| acc.max(f)))
            .map(Value::Float)
    }
    fn min(args: &[Value]) -> Result<Value, String> {
        args.iter()
            .try_fold(f64::INFINITY, |acc, v| v.to_float().map(|f| acc.min(f)))
            .map(Value::Float)
    }

    // ---- I/O Functions Implementation ----
    fn print(args: &[Value]) -> Result<Value, String> {
        let output = args
            .iter()
            .map(|v| v.display())
            .collect::<Vec<_>>()
            .join(" ");
        print!("{}", output);
        Ok(Value::Null)
    }
    fn println(args: &[Value]) -> Result<Value, String> {
        let output = args
            .iter()
            .map(|v| v.display())
            .collect::<Vec<_>>()
            .join(" ");
        println!("{}", output);
        Ok(Value::Null)
    }

    fn input(args: &[Value]) -> Result<Value, String> {
        // Print a prompt if there are any arguments
        if !args.is_empty() {
            let prompt = args
                .iter()
                .map(|v| v.display())
                .collect::<Vec<_>>()
                .join(" ");
            print!("{}", prompt);
            io::stdout().flush().map_err(|e| e.to_string())?; // Ensure the prompt is printed immediately
        }

        // Create a mutable String to hold the user input
        let mut input_string = String::new();

        // Read user input
        io::stdin()
            .read_line(&mut input_string)
            .map_err(|e| e.to_string())?;

        // Trim the input to remove any trailing newline characters
        let trimmed_input = input_string.trim();

        // Return the input
        Ok(Value::String(trimmed_input.to_string()))
    }

    // ---- Type Conversion Functions Implementation ----
    fn cast_to_int(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Integer(args[0].to_integer()?))
    }
    fn cast_to_float(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Float(args[0].to_float()?))
    }
    fn cast_to_bool(args: &[Value]) -> Result<Value, String> {
        Ok(Value::Boolean(args[0].to_bool()?))
    }
    fn cast_to_string(args: &[Value]) -> Result<Value, String> {
        Ok(Value::String(args[0].display()))
    }
}
