// Copyright (c) 2026 bazelik-null

use crate::morsel_interpreter::environment::morsel_std::BuiltinFunctionDispatcher;
use crate::morsel_interpreter::environment::symbol_table::SymbolTable;
use crate::morsel_interpreter::environment::symbol_table::function_symbol::FunctionSymbol;
use crate::morsel_interpreter::environment::symbol_table::variable_symbol::VariableSymbol;
use crate::morsel_interpreter::environment::types::Type;
use crate::morsel_interpreter::environment::value::Value;
use crate::morsel_interpreter::parser::ast_node::Node;

/// Executor which evaluates AST using only SymbolTable
pub struct Executor {
    symbol_table: SymbolTable,
}

impl Executor {
    pub fn new(symbol_table: SymbolTable) -> Self {
        let mut executor = Executor { symbol_table };
        executor.symbol_table.push_scope(); // Global scope
        executor
    }

    /// Main entry point. Executes the program by calling main()
    pub fn execute(&mut self) -> Result<(), String> {
        // Look for main function in symbol table
        let function = self
            .symbol_table
            .functions
            .lookup("main")
            .ok_or_else(|| format!("Function '{}' not found", "main"))?
            .clone();

        // Call main()
        self.evaluate_function("main", &function, &[])?;

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
            Node::Reference(name) => self.get_variable(name),
            Node::Block(statements) => self.eval_block(statements),
            Node::LetBinding {
                reference: name,
                value,
                type_annotation,
            } => self.eval_let(name.clone(), value, type_annotation),
            Node::Assignment { name, value } => self.eval_assignment(name, value),
            Node::Call { name, args } => self.eval_call(name, args, target_type),
            Node::FuncBinding() => Ok(Value::Null),
        }
    }

    /// Get a variable value from the symbol table
    fn get_variable(&self, name: &str) -> Result<Value, String> {
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

        let result_type = result.type_of();

        // Check if types are compatible (same type or numeric compatibility)
        if !self.is_type_compatible(result_type, *type_annotation) {
            return Err(format!(
                "Type mismatch in let binding '{}': expected {}, got {}",
                name, type_annotation, result_type
            ));
        }

        // Coerce to target type if needed
        let coerced_value = self.coerce_value(result, *type_annotation)?;

        // Create variable symbol in current scope
        let var_symbol = VariableSymbol::new(
            name.clone(),
            *type_annotation,
            false,
            self.symbol_table.depth(),
        );

        // Define and set value in one operation
        self.symbol_table
            .variables
            .define_with_value(var_symbol, coerced_value.clone())?;

        Ok(coerced_value)
    }

    /// Evaluate an assignment with type checking
    fn eval_assignment(&mut self, name: &str, value: &Node) -> Result<Value, String> {
        // Get variable metadata
        if !self.symbol_table.variables.exists(name) {
            return Err(format!("Variable '{}' not found", name));
        }

        // Get the expected type from the variable symbol
        let var_symbol = self
            .symbol_table
            .variables
            .lookup(name)
            .ok_or_else(|| format!("Variable '{}' symbol not found", name))?;

        let expected_type = var_symbol.type_annotation;

        // Evaluate assignment value with target type context
        let result = self.eval_with_type(value, Some(expected_type))?;

        let result_type = result.type_of();

        // Check if types are compatible
        if !self.is_type_compatible(result_type, expected_type) {
            return Err(format!(
                "Type mismatch in assignment to '{}': expected {}, got {}",
                name, expected_type, result_type
            ));
        }

        // Coerce to target type if needed
        let coerced_value = self.coerce_value(result, expected_type)?;

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
            .lookup(name)
            .ok_or_else(|| format!("Function '{}' not found", name))?
            .clone();

        // Validate arguments using the function table
        self.symbol_table
            .functions
            .validate_arg_count(name, args.len())?;

        if function.is_builtin() {
            return self.evaluate_builtin_function(
                function.name.as_str(),
                function.namespace.as_str(),
                args,
                target_type,
            );
        }

        self.evaluate_function(name, &function, args)
    }

    /// Check if two types are compatible
    fn is_type_compatible(&self, from: Type, to: Type) -> bool {
        if from == to {
            return true;
        }

        // Allow numeric type compatibility
        matches!(
            (from, to),
            (Type::Integer, Type::Float)
                | (Type::Float, Type::Integer)
                | (Type::Integer, Type::Integer)
                | (Type::Float, Type::Float)
        )
    }

    /// Coerce a value to the target type
    fn coerce_value(&self, value: Value, target_type: Type) -> Result<Value, String> {
        let value_type = value.type_of();

        if value_type == target_type {
            return Ok(value);
        }

        match (value, target_type) {
            (Value::Integer(i), Type::Float) => Ok(Value::Float(i as f64)),
            (Value::Float(f), Type::Integer) => Ok(Value::Integer(f as i64)),
            (v, _) => Err(format!("Cannot coerce {} to {}", v.type_of(), target_type)),
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
        // If target type is specified and is numeric, use it
        if let Some(Type::Integer | Type::Float) = target_type {
            return target_type.unwrap();
        }

        // Otherwise, preserve integer type when both operands are integers
        if left.type_of() == Type::Integer && right.type_of() == Type::Integer {
            Type::Integer
        } else {
            Type::Float
        }
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

    /// Evaluate a user-defined function with explicit type checking
    fn evaluate_function(
        &mut self,
        name: &str,
        function: &FunctionSymbol,
        args: &[Value],
    ) -> Result<Value, String> {
        // Get function implementation
        let impl_node = function
            .implementation
            .as_ref()
            .ok_or_else(|| format!("Undefined implementation for function: {}", name))?
            .clone();

        // Push function scope
        self.symbol_table.push_scope();

        // Bind parameters with explicit type checking
        for (param, arg_value) in function.parameters.iter().zip(args.iter()) {
            let arg_type = arg_value.type_of();

            // Check type compatibility
            if !self.is_type_compatible(arg_type, param.type_annotation) {
                self.symbol_table.pop_scope();
                return Err(format!(
                    "Function '{}' parameter '{}' type mismatch: expected {}, got {}",
                    name, param.name, param.type_annotation, arg_type
                ));
            }

            // Coerce argument to parameter type if needed
            let coerced_arg = self.coerce_value(arg_value.clone(), param.type_annotation)?;

            // Create parameter symbol in current scope and set value
            let param_symbol = VariableSymbol::new(
                param.name.clone(),
                param.type_annotation,
                false, // Parameters are immutable
                self.symbol_table.depth(),
            );

            self.symbol_table
                .variables
                .define_with_value(param_symbol, coerced_arg)?;
        }

        // Evaluate function body
        let result = self.eval(&impl_node);

        // Pop function scope
        self.symbol_table.pop_scope();

        result
    }
}
