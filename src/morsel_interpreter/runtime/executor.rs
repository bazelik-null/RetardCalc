// Copyright (c) 2026 bazelik-null

use crate::morsel_interpreter::environment::morsel_std::BuiltinFunctionDispatcher;
use crate::morsel_interpreter::environment::symbol_table::SymbolTable;
use crate::morsel_interpreter::environment::symbol_table::function_symbol::FunctionMetadata;
use crate::morsel_interpreter::environment::types::Type;
use crate::morsel_interpreter::environment::value::Value;
use crate::morsel_interpreter::parser::ast_node::Node;

/// Executor which evaluates AST using only SymbolTable
pub struct Executor<'a> {
    symbol_table: &'a mut SymbolTable,
}

impl<'a> Executor<'a> {
    pub fn new(symbol_table: &'a mut SymbolTable) -> Self {
        symbol_table.push_scope(); // Global scope
        Executor { symbol_table }
    }

    /// Main entry point. Executes the program by calling main()
    pub fn execute(&mut self) -> Result<(), String> {
        // Call main()
        self.evaluate_function("main", &[])?;

        Ok(())
    }

    /// Evaluate an AST node without type context
    fn eval(&mut self, node: &Node) -> Result<Value, String> {
        self.eval_with_type(node, None)
    }

    /// Evaluate an AST node with optional target type context
    fn eval_with_type(&mut self, node: &Node, target_type: Option<Type>) -> Result<Value, String> {
        match node {
            Node::Literal(value) => Ok(value.clone()),
            Node::Reference(name) => self.get_variable(name).cloned(),
            Node::Block(statements) => self.eval_block(statements),
            Node::LetBinding {
                reference: name,
                value,
                type_annotation,
            } => self.eval_let(name.parse().unwrap(), value, type_annotation),
            Node::Assignment { name, value } => self.eval_assignment(name, value),
            Node::Call { name, args } => self.eval_call(name, args, target_type),
            Node::FuncBinding() => Ok(Value::Null),
        }
    }

    /// Get a variable value from the symbol table
    fn get_variable(&self, name: &str) -> Result<&Value, String> {
        self.symbol_table
            .variables
            .get_value(name)
            .ok_or_else(|| format!("Variable '{}' not initialized", name))
    }

    /// Evaluate a block of statements
    fn eval_block(&mut self, statements: &[Node]) -> Result<Value, String> {
        self.symbol_table.push_scope();

        let mut result = Value::Null;
        for stmt in statements {
            result = self.eval(stmt)?;
        }

        self.symbol_table.pop_scope();

        Ok(result)
    }

    /// Evaluate a let binding with explicit type coercion
    fn eval_let(
        &mut self,
        name: String,
        value: &Node,
        type_annotation: &Type,
    ) -> Result<Value, String> {
        // Evaluate the expression with target type context
        let result = self.eval_with_type(value, Some(*type_annotation))?;

        // Coerce to target type if needed
        let coerced_value = self.coerce_value(&result, *type_annotation)?;

        // Create variable symbol in current scope
        self.symbol_table.variables.define_with_value(
            name.clone(),
            *type_annotation,
            false,
            coerced_value.clone(),
        )?;

        Ok(coerced_value)
    }

    /// Evaluate an assignment with type checking
    fn eval_assignment(&mut self, name: &str, value: &Node) -> Result<Value, String> {
        // Get the expected type from the variable symbol
        let var_symbol = self
            .symbol_table
            .variables
            .lookup(name)
            .ok_or_else(|| format!("Variable '{}' not found", name))?;

        let expected_type = var_symbol.type_annotation;

        // Evaluate assignment value with target type context
        let result = self.eval_with_type(value, Some(expected_type))?;

        let result_type = result.type_of();

        // Check if types are compatible
        if !result_type.is_compatible_with(&expected_type) {
            return Err(format!(
                "Type mismatch in assignment to '{}': expected {}, got {}",
                name, expected_type, result_type
            ));
        }

        // Coerce to target type if needed
        let coerced_value = self.coerce_value(&result, expected_type)?;

        // Update value in symbol table
        self.symbol_table
            .variables
            .set_value(name, coerced_value.clone())?;

        Ok(coerced_value)
    }

    /// Evaluate a function call or operator with optional target type
    fn eval_call(
        &mut self,
        name: &str,
        args: &[Node],
        target_type: Option<Type>,
    ) -> Result<Value, String> {
        // Get argument values
        let values: Result<Vec<Value>, String> = args.iter().map(|arg| self.eval(arg)).collect();
        let values = values?;

        self.dispatch_call(name, &values, target_type)
    }

    /// Dispatch function calls and operators with target type context
    fn dispatch_call(
        &mut self,
        name: &str,
        values: &[Value],
        target_type: Option<Type>,
    ) -> Result<Value, String> {
        match name {
            // Binary operators
            "+" | "-" | "*" | "/" | "^" | "%" if values.len() == 2 => {
                self.apply_binary(name, &values[0], &values[1], target_type)
            }
            // Unary operators
            "-" if values.len() == 1 => self.apply_unary_minus(&values[0], target_type),
            // Everything else (function calls)
            _ => self.apply_function(name, values, target_type),
        }
    }

    /// Validates arguments and applies either builtin or user function
    fn apply_function(
        &mut self,
        name: &str,
        args: &[Value],
        target_type: Option<Type>,
    ) -> Result<Value, String> {
        // Look for function in symbol table
        let function = self
            .symbol_table
            .functions
            .lookup_with_namespace(name, None)
            .ok_or_else(|| format!("Function '{}' not found", name))?;

        // Execute builtin or user-defined function
        if function.metadata.is_builtin {
            self.evaluate_builtin_function(
                &function.metadata.name,
                &function.metadata.namespace,
                args,
                target_type,
            )
        } else {
            self.evaluate_function(name, args)
        }
    }

    /// Coerce a value to the target type
    fn coerce_value(&self, value: &Value, target_type: Type) -> Result<Value, String> {
        let value_type = value.type_of();

        // Fast path: no coercion needed
        if value_type == target_type {
            return Ok(value.clone());
        }

        // Coercion paths
        match (&value, target_type) {
            (Value::Integer(i), Type::Float) => Ok(Value::Float(*i as f64)),
            (Value::Float(f), Type::Integer) => Ok(Value::Integer(*f as i64)),
            _ => Err(format!("Cannot coerce {} to {}", value_type, target_type)),
        }
    }

    /// Validate that both operands are numeric
    fn validate_numeric_operands(
        &self,
        op: &str,
        left: &Value,
        right: &Value,
    ) -> Result<(), String> {
        let left_type = left.type_of();
        let right_type = right.type_of();

        if !matches!(left_type, Type::Integer | Type::Float) {
            return Err(format!(
                "Operator '{}' requires numeric operands, got {} on left",
                op, left_type
            ));
        }
        if !matches!(right_type, Type::Integer | Type::Float) {
            return Err(format!(
                "Operator '{}' requires numeric operands, got {} on right",
                op, right_type
            ));
        }
        Ok(())
    }

    /// Apply a binary operator with target type context
    fn apply_binary(
        &self,
        op: &str,
        left: &Value,
        right: &Value,
        target_type: Option<Type>,
    ) -> Result<Value, String> {
        // Handle string concatenation
        if op == "+" && matches!(left, Value::String(_)) || matches!(right, Value::String(_)) {
            let left_str = left.to_string();
            let right_str = right.to_string();
            return Ok(Value::String(format!("{}{}", left_str, right_str)));
        }

        self.validate_numeric_operands(op, left, right)?;

        let left_f = left.to_float()?;
        let right_f = right.to_float()?;

        let result = match op {
            "+" => left_f + right_f,
            "-" => left_f - right_f,
            "*" => left_f * right_f,
            "/" => {
                if right_f == 0.0 {
                    return Err("Division by zero".to_string());
                }
                left_f / right_f
            }
            "^" => left_f.powf(right_f),
            "%" => left_f.rem_euclid(right_f),
            _ => unreachable!(),
        };

        // Determine output type based on operands and target type
        let output_type = self.determine_numeric_type(left, right, target_type);

        match output_type {
            Type::Integer => Ok(Value::Integer(result as i64)),
            Type::Float => Ok(Value::Float(result)),
            _ => unreachable!(),
        }
    }

    /// Determine the numeric output type
    fn determine_numeric_type(
        &self,
        left: &Value,
        right: &Value,
        target_type: Option<Type>,
    ) -> Type {
        target_type
            .filter(|t| matches!(t, Type::Integer | Type::Float))
            .unwrap_or_else(|| {
                if left.type_of() == Type::Integer && right.type_of() == Type::Integer {
                    Type::Integer
                } else {
                    Type::Float
                }
            })
    }

    /// Apply unary minus with target type context
    fn apply_unary_minus(
        &self,
        operand: &Value,
        target_type: Option<Type>,
    ) -> Result<Value, String> {
        match operand {
            Value::Float(f) => {
                let result = Value::Float(-f);
                if let Some(Type::Integer) = target_type {
                    Ok(Value::Integer(-(*f as i64)))
                } else {
                    Ok(result)
                }
            }
            Value::Integer(i) => {
                let result = Value::Integer(-i);
                if let Some(Type::Float) = target_type {
                    Ok(Value::Float(-(*i as f64)))
                } else {
                    Ok(result)
                }
            }
            _ => Err(format!(
                "Unary minus requires numeric operand, got {}",
                operand.type_of()
            )),
        }
    }

    /// Evaluates a builtin function with target type context
    fn evaluate_builtin_function(
        &self,
        name: &str,
        namespace: &str,
        args: &[Value],
        target_type: Option<Type>,
    ) -> Result<Value, String> {
        // Call the builtin implementation
        let result = BuiltinFunctionDispatcher::call(name, namespace, args)?;

        // Handle target type conversion for math functions
        match (&result, target_type) {
            (Value::Float(f), Some(Type::Integer)) => Ok(Value::Integer(*f as i64)),
            _ => Ok(result),
        }
    }

    /// Evaluate a user-defined function
    fn evaluate_function(&mut self, fq_name: &str, args: &[Value]) -> Result<Value, String> {
        // Extract and clone what we need
        let (metadata, implementation) = {
            let function = self
                .symbol_table
                .functions
                .lookup(fq_name)
                .ok_or_else(|| format!("Function '{}' not found", fq_name))?;

            if function.implementation.is_none() {
                return Err(format!("Function '{}' has no implementation", fq_name));
            }

            (function.metadata.clone(), function.implementation.clone())
        };

        self.symbol_table.push_scope();

        if let Err(e) = self.bind_parameters(&metadata, args) {
            self.symbol_table.pop_scope();
            return Err(e);
        }

        let result = self.eval(implementation.as_ref().unwrap());
        self.symbol_table.pop_scope();
        result
    }

    /// Bind function parameters
    fn bind_parameters(
        &mut self,
        metadata: &FunctionMetadata,
        args: &[Value],
    ) -> Result<(), String> {
        for (param, arg_value) in metadata.parameters.iter().zip(args.iter()) {
            let coerced_arg = if arg_value.type_of() == param.type_annotation {
                arg_value.clone()
            } else {
                self.coerce_value(arg_value, param.type_annotation)?
            };

            self.symbol_table.variables.define_with_value(
                param.name.clone(),
                param.type_annotation,
                false,
                coerced_arg,
            )?;
        }

        Ok(())
    }
}
