// Copyright (c) 2026 bazelik-null

use crate::morsel_core::evaluating::functions::{FunctionTable, ReturnType};
use crate::morsel_core::parsing::node::Node;
use std::collections::HashMap;
use std::sync::Arc;

/// Evaluates an AST and returns the result
pub struct Evaluator {
    func_table: Arc<FunctionTable>,
    variables: HashMap<String, f64>,
}

impl Evaluator {
    /// Create a new evaluator with a function table
    pub fn new(func_table: Arc<FunctionTable>) -> Self {
        Evaluator {
            func_table,
            variables: HashMap::new(),
        }
    }

    /// Evaluate an AST node
    pub fn eval(&mut self, node: &Node) -> Result<f64, String> {
        match node {
            Node::Literal(value) => Ok(*value),
            Node::Variable(name) => self.get_variable(name),
            Node::Block(statements) => self.eval_block(statements),
            Node::Let { name, value } => self.eval_let(name, value),
            Node::Call { name, args } => self.eval_call(name, args),
        }
    }

    /// Set a variable value
    pub fn set_variable(&mut self, name: String, value: f64) {
        self.variables.insert(name, value);
    }

    /// Get a variable value
    pub fn get_variable(&self, name: &str) -> Result<f64, String> {
        self.variables
            .get(name)
            .copied()
            .ok_or_else(|| format!("Undefined variable: '{}'", name))
    }

    /// Evaluate a block of statements, returning the last result
    fn eval_block(&mut self, statements: &[Node]) -> Result<f64, String> {
        statements.iter().try_fold(0.0, |_, stmt| self.eval(stmt))
    }

    /// Evaluate a let binding
    fn eval_let(&mut self, name: &str, value: &Node) -> Result<f64, String> {
        let result = self.eval(value)?;
        self.set_variable(name.to_string(), result);
        Ok(result)
    }

    /// Evaluate a function call or operator
    fn eval_call(&mut self, name: &str, args: &[Node]) -> Result<f64, String> {
        let values: Result<Vec<f64>, String> = args.iter().map(|arg| self.eval(arg)).collect();
        let values = values?;

        match name {
            "+" | "-" | "*" | "/" | "^" | "%" if values.len() == 2 => {
                self.apply_binary(name, values[0], values[1])
            }
            "-" if values.len() == 1 => Ok(-values[0]),
            _ => self.apply_function(name, &values),
        }
    }

    /// Validates arguments and applies either builtin or user function (user functions are not implemented at the moment btw)
    fn apply_function(&mut self, name: &str, args: &[f64]) -> Result<f64, String> {
        if self.func_table.is_builtin(name) {
            self.func_table.validate_args(name, args.len())?;
            return match self.apply_builtin(name, args)? {
                ReturnType::Value(v) => Ok(v),
                ReturnType::Void => Ok(0.0),
            };
        }

        self.apply_user_function(name, args)
    }

    /// Apply a binary operator
    fn apply_binary(&self, op: &str, left: f64, right: f64) -> Result<f64, String> {
        Ok(match op {
            "+" => left + right,
            "-" => left - right,
            "*" => left * right,
            "/" => left / right,
            "^" => left.powf(right),
            "%" => left.rem_euclid(right),
            _ => return Err(format!("Unknown binary operator: '{}'", op)),
        })
    }

    /// Apply a builtin function
    fn apply_builtin(&self, func: &str, args: &[f64]) -> Result<ReturnType, String> {
        let result = match func {
            // Single-argument math functions
            "sqrt" => ReturnType::Value(args[0].sqrt()),
            "cbrt" => ReturnType::Value(args[0].cbrt()),
            "ln" => ReturnType::Value(args[0].ln()),
            "sin" => ReturnType::Value(args[0].sin()),
            "cos" => ReturnType::Value(args[0].cos()),
            "tan" => ReturnType::Value(args[0].tan()),
            "asin" => ReturnType::Value(args[0].asin()),
            "acos" => ReturnType::Value(args[0].acos()),
            "atan" => ReturnType::Value(args[0].atan()),
            "abs" => ReturnType::Value(args[0].abs()),
            "round" => ReturnType::Value(args[0].round()),
            "floor" => ReturnType::Value(args[0].floor()),
            "ceil" => ReturnType::Value(args[0].ceil()),

            // Multi-argument functions
            "root" => ReturnType::Value(args[0].powf(1.0 / args[1])),
            "log" => ReturnType::Value(args[1].log(args[0])),
            "min" => ReturnType::Value(args.iter().copied().fold(f64::INFINITY, f64::min)),
            "max" => ReturnType::Value(args.iter().copied().fold(f64::NEG_INFINITY, f64::max)),

            // I/O functions
            "print" => {
                println!(
                    "{}",
                    args.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                ReturnType::Void
            }

            _ => return Err(format!("Unknown function: '{}'", func)),
        };

        Ok(result)
    }

    /// Apply a user-defined function
    fn apply_user_function(&mut self, func: &str, args: &[f64]) -> Result<f64, String> {
        self.func_table.validate_args(func, args.len())?;

        let function = self
            .func_table
            .get_function_owned(func)
            .ok_or_else(|| format!("Unknown function: '{}'", func))?;

        let impl_node = function
            .implementation
            .as_ref()
            .ok_or(format!("Undefined implementation for function: {}", func))?;

        // Save and restore variables for scope isolation
        let saved_vars = std::mem::take(&mut self.variables);
        let result = self.eval(impl_node);
        self.variables = saved_vars;

        result
    }
}
