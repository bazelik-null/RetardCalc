use crate::core::compiler::parser::symbol::{ScopeStack, Symbol};
use crate::core::compiler::parser::tree::Node;
use crate::core::compiler::parser::type_inference::{
    infer_binary_type, infer_literal_type, infer_unary_type, types_compatible,
};
use crate::core::compiler::preprocessor::token::LiteralValue;
use crate::core::shared::types::Type;
use lasso::{Rodeo, Spur};
use std::collections::HashMap;

pub struct SemanticAnalyzer<'a> {
    rodeo: &'a Rodeo,
    scope_stack: ScopeStack, // Tracks variable visibility across scopes
    errors: Vec<String>,
    functions: HashMap<Spur, (Vec<Type>, Option<Type>)>, // Function signatures: (params, return_type)
    current_return_type: Option<Type>,                   // Expected return type in current function
    current_function: Option<Spur>, // Track current function name for error context
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(rodeo: &'a Rodeo) -> Self {
        Self {
            rodeo,
            scope_stack: ScopeStack::new(),
            errors: Vec::new(),
            functions: HashMap::new(),
            current_return_type: None,
            current_function: None,
        }
    }

    pub fn analyze(&mut self, nodes: &mut [Node]) -> Result<(), Vec<String>> {
        // Gather all function signatures
        self.collect_all_declarations(nodes);

        // Validate each node
        nodes.iter_mut().for_each(|n| {
            let _ = self.analyze_node(n);
        });

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    // Recursively collect all function declarations from AST
    fn collect_all_declarations(&mut self, nodes: &[Node]) {
        nodes.iter().for_each(|n| self.collect_declarations(n));
    }

    // Extract function signatures
    fn collect_declarations(&mut self, node: &Node) {
        match node {
            Node::FunctionDecl {
                name,
                params,
                return_type,
                ..
            } => {
                let param_types = params.iter().map(|p| p.type_annotation.clone()).collect();
                self.functions
                    .insert(*name, (param_types, return_type.clone()));
            }

            // Recurse into blocks to find nested functions
            Node::Block(nodes) => nodes.iter().for_each(|n| self.collect_declarations(n)),
            _ => {}
        }
    }

    fn analyze_node(&mut self, node: &mut Node) -> Result<Type, ()> {
        match node {
            Node::Literal(lit) => Ok(infer_literal_type(lit)),
            Node::Identifier(name) => self.lookup_identifier(*name),
            Node::VariableDecl { .. } => self.analyze_var_decl(node),
            Node::Assignment { .. } => self.analyze_assignment(node),
            Node::Binary { .. } => self.analyze_binary(node),
            Node::Unary { .. } => self.analyze_unary(node),
            Node::If { .. } => self.analyze_if(node),
            Node::While { .. } => self.analyze_while(node),
            Node::Block(nodes) => self.analyze_block(nodes),
            Node::FunctionDecl { .. } => self.analyze_func_decl(node),
            Node::FunctionCall { .. } => self.analyze_func_call(node),
            Node::SysCall { .. } => self.analyze_syscall(node),
            Node::Return(val) => self.analyze_return(val),
            Node::ArrayLiteral(elements) => self.analyze_array_literal(elements),
            Node::ArrayAccess { array, index } => self.analyze_array_access(array, index),
            Node::Reference {
                value: inner,
                mutable,
            } => self.analyze_reference(inner, *mutable),
            Node::Dereference(inner) => self.analyze_dereference(inner),
        }
    }

    fn is_lvalue(&self, node: &Node) -> bool {
        matches!(
            node,
            Node::Identifier(_) | Node::ArrayAccess { .. } | Node::Dereference(_) // Dereferenced pointers are lvalues
        )
    }

    fn validate_lvalue(&mut self, node: &Node) -> Result<(), ()> {
        if !self.is_lvalue(node) {
            self.error(format!(
                "TypeError: Cannot take reference of non-lvalue: {:?}",
                node
            ));
            Err(())
        } else {
            Ok(())
        }
    }

    fn lookup_identifier(&mut self, name: Spur) -> Result<Type, ()> {
        self.scope_stack
            .lookup(name)
            .map(|s| s.type_annotation.clone())
            .ok_or_else(|| {
                self.error(format!(
                    "NameError: Undefined variable: {}",
                    self.resolve_name(&name)
                ));
            })
    }

    fn analyze_reference(&mut self, inner: &mut Box<Node>, mutable: bool) -> Result<Type, ()> {
        // Validate lvalue first
        self.validate_lvalue(inner)?;

        let inner_type = self.analyze_node(inner)?;

        // Check mutability constraints
        if mutable {
            // Reject mutable ref to immutable ref
            if matches!(inner_type, Type::Reference(_)) {
                self.error(
                    "TypeError: Cannot create mutable reference to immutable reference".to_string(),
                );
                return Err(());
            }

            // Mutable references require the target to be mutable
            if let Node::Identifier(name) = inner.as_ref()
                && let Some(symbol) = self.scope_stack.lookup(*name)
                && !symbol.mutable
            {
                self.error(format!(
                    "TypeError: Cannot create mutable reference to immutable variable: {}",
                    self.resolve_name(name)
                ));
                return Err(());
            }
            Ok(Type::MutableReference(Box::new(inner_type)))
        } else {
            Ok(Type::Reference(Box::new(inner_type)))
        }
    }

    fn analyze_dereference(&mut self, inner: &mut Box<Node>) -> Result<Type, ()> {
        let inner_type = self.analyze_node(inner)?;

        match inner_type {
            Type::Reference(pointee) | Type::MutableReference(pointee) => {
                // Strip one level of reference
                Ok(*pointee)
            }
            _ => {
                self.error(format!(
                    "TypeError: Cannot dereference non-reference type: {}",
                    inner_type
                ));
                Err(())
            }
        }
    }

    fn analyze_array_literal(&mut self, elements: &mut [Node]) -> Result<Type, ()> {
        // Empty array has unknown element type
        if elements.is_empty() {
            return Ok(Type::Array(Box::new(Type::Void)));
        }

        let first_type = self.analyze_node(&mut elements[0])?;

        // Validate all elements have same type
        for (i, elem) in elements.iter_mut().enumerate().skip(1) {
            let elem_type = self.analyze_node(elem)?;
            if !types_compatible(&elem_type, &first_type) {
                self.error(format!(
                    "TypeError: Array element type mismatch at index {}: expected {}, got {}",
                    i, first_type, elem_type
                ));
                return Err(());
            }
        }

        // Infer FixedArray from literal
        Ok(Type::FixedArray(Box::new(first_type), elements.len()))
    }

    fn analyze_array_access(&mut self, array: &mut Node, index: &mut Node) -> Result<Type, ()> {
        let array_type = self.analyze_node(array)?;

        // Index must be integer
        let index_type = self.analyze_node(index)?;
        if index_type != Type::Integer {
            self.error(format!(
                "TypeError: Array index must be integer, got {}",
                index_type
            ));
            return Err(());
        }

        match &array_type {
            Type::FixedArray(inner_type, size) => {
                // Validate constant indices at compile time
                if let Node::Literal(LiteralValue::Integer(idx)) = index
                    && (*idx < 0 || (*idx as usize) >= *size)
                {
                    self.error(format!(
                        "OutOfBounds: Array index {} out of bounds for array of size {}",
                        idx, size
                    ));
                    return Err(());
                }
                Ok((**inner_type).clone())
            }
            Type::Array(inner_type) => {
                // We can't check dynamic arrays at compile time
                if let Node::Literal(LiteralValue::Integer(idx)) = index
                    && (*idx < 0)
                {
                    self.error(format!(
                        "OutOfBounds: Array index {} is negative and will always be out of bounds",
                        idx
                    ));
                    return Err(());
                }
                Ok((**inner_type).clone())
            }
            _ => {
                self.error(format!(
                    "TypeError: Cannot index non-array type: {}",
                    array_type
                ));
                Err(())
            }
        }
    }

    fn analyze_var_decl(&mut self, node: &mut Node) -> Result<Type, ()> {
        let (name, mutable, type_annotation, value) = match node {
            Node::VariableDecl {
                name,
                mutable,
                type_annotation,
                value,
            } => (name, mutable, type_annotation, value),
            _ => return Err(()),
        };

        let value_type = self.analyze_node(value)?; // Get type of assigned value

        // Use annotation if present, else infer from value
        let mut declared_type = type_annotation
            .get_or_insert_with(|| value_type.clone())
            .clone();

        // Convert inferred FixedArray to dynamic Array
        if type_annotation.is_none() {
            declared_type = match declared_type {
                Type::FixedArray(inner, _) => Type::Array(inner),
                other => other,
            };
        }

        // Ensure value type matches declared type
        if !types_compatible(&value_type, &declared_type) {
            self.error(format!(
                "TypeError: Type mismatch: expected {}, got {}",
                declared_type, value_type
            ));
            return Err(());
        }

        // Add symbol to current scope
        self.scope_stack.define(Symbol {
            name: *name,
            type_annotation: declared_type.clone(),
            mutable: *mutable,
            scope_level: self.scope_stack.current_level(),
        });

        Ok(declared_type)
    }

    fn analyze_assignment(&mut self, node: &mut Node) -> Result<Type, ()> {
        let (target, value) = match node {
            Node::Assignment { target, value } => (target, value),
            _ => return Err(()),
        };

        let value_type = self.analyze_node(value)?;

        match target.as_mut() {
            Node::Dereference(ref_expr) => {
                let ref_type = self.analyze_node(ref_expr)?;

                match ref_type {
                    Type::MutableReference(pointee) => {
                        if !types_compatible(&value_type, &pointee) {
                            self.error(format!(
                                "TypeError: Type mismatch in dereferenced assignment: expected {}, got {}",
                                pointee, value_type
                            ));
                            return Err(());
                        }
                        Ok(*pointee)
                    }
                    Type::Reference(_) => {
                        self.error(
                            "TypeError: Cannot assign through immutable reference".to_string(),
                        );
                        Err(())
                    }
                    _ => {
                        self.error(format!(
                            "TypeError: Cannot dereference for assignment: {}",
                            ref_type
                        ));
                        Err(())
                    }
                }
            }

            Node::Identifier(name) => {
                let symbol = self.scope_stack.lookup(*name).ok_or_else(|| {
                    self.error(format!(
                        "NameError: Undefined variable: {}",
                        self.resolve_name(name)
                    ));
                })?;

                if !symbol.mutable {
                    self.error(format!(
                        "TypeError: Cannot assign to immutable variable: {}",
                        self.resolve_name(name)
                    ));
                    return Err(());
                }

                if !types_compatible(&value_type, &symbol.type_annotation) {
                    self.error(format!(
                        "TypeError: Type mismatch in assignment: expected {}, got {}",
                        symbol.type_annotation, value_type
                    ));
                    return Err(());
                }

                Ok(symbol.type_annotation.clone())
            }

            Node::ArrayAccess { array, index } => {
                if let Node::Identifier(arr_name) = array.as_ref() {
                    let symbol = self.scope_stack.lookup(*arr_name).ok_or_else(|| {
                        self.error(format!(
                            "NameError: Undefined variable: {}",
                            self.resolve_name(arr_name)
                        ));
                    })?;

                    if !symbol.mutable {
                        self.error(format!(
                            "TypeError: Cannot mutate immutable array: {}",
                            self.resolve_name(arr_name)
                        ));
                        return Err(());
                    }
                }

                let element_type = self.analyze_array_access(array, index)?;

                if !types_compatible(&value_type, &element_type) {
                    self.error(format!(
                        "TypeError: Type mismatch in array assignment: expected {}, got {}",
                        element_type, value_type
                    ));
                    return Err(());
                }

                Ok(element_type)
            }

            _ => {
                self.error("SyntaxError: Invalid assignment target".to_string());
                Err(())
            }
        }
    }

    fn analyze_binary(&mut self, node: &mut Node) -> Result<Type, ()> {
        let (lhs, op, rhs) = match node {
            Node::Binary { lhs, op, rhs } => (lhs, op, rhs),
            _ => return Err(()),
        };

        let lhs_type = self.analyze_node(lhs)?;
        let rhs_type = self.analyze_node(rhs)?;

        // Delegate to type inference for operator
        infer_binary_type(&lhs_type, op, &rhs_type, &mut self.errors)
    }

    fn analyze_unary(&mut self, node: &mut Node) -> Result<Type, ()> {
        let (op, rhs) = match node {
            Node::Unary { op, rhs } => (op, rhs),
            _ => return Err(()),
        };

        let rhs_type = self.analyze_node(rhs)?;

        // Delegate to type inference for operator
        infer_unary_type(op, &rhs_type, &mut self.errors)
    }

    fn analyze_if(&mut self, node: &mut Node) -> Result<Type, ()> {
        let (condition, then_branch, else_branch) = match node {
            Node::If {
                condition,
                then_branch,
                else_branch,
            } => (condition, then_branch, else_branch),
            _ => return Err(()),
        };

        let cond_type = self.analyze_node(condition)?;
        // Condition must be boolean
        if cond_type != Type::Boolean {
            self.error(format!(
                "TypeError: If condition must be boolean, got {}",
                cond_type
            ));
            return Err(());
        }

        // Analyze then branch in new scope
        self.scope_stack.push();
        let then_type = self.analyze_node(then_branch)?;
        self.scope_stack.pop();

        // Analyze else branch (if present) in new scope
        let else_type = if let Some(else_b) = else_branch {
            self.scope_stack.push();
            let t = self.analyze_node(else_b)?;
            self.scope_stack.pop();
            Some(t)
        } else {
            None
        };

        // Both branches type must match if both exist
        match (else_type, then_type.clone()) {
            (Some(else_t), then_t) => {
                if types_compatible(&else_t, &then_t) {
                    Ok(then_t)
                } else {
                    self.error(format!(
                        "TypeError: If/else branches have incompatible types: {} vs {}",
                        then_t, else_t
                    ));
                    Err(())
                }
            }
            (None, then_t) => Ok(then_t), // If no else, return then type
        }
    }

    fn analyze_while(&mut self, node: &mut Node) -> Result<Type, ()> {
        let (condition, body) = match node {
            Node::While { condition, body } => (condition, body),
            _ => return Err(()),
        };

        let cond_type = self.analyze_node(condition)?;
        // Condition must be boolean
        if cond_type != Type::Boolean {
            self.error(format!(
                "TypeError: While condition must be boolean, got {}",
                cond_type
            ));
            return Err(());
        }

        // Analyze body in new scope
        self.scope_stack.push();
        self.analyze_node(body)?;
        self.scope_stack.pop();

        Ok(Type::Void) // Loops return void
    }

    fn analyze_block(&mut self, nodes: &mut [Node]) -> Result<Type, ()> {
        self.scope_stack.push(); // Enter new scope

        let nodes_len = nodes.len();
        let mut last_type = Type::Void; // Default to void

        for (i, node) in nodes.iter_mut().enumerate() {
            let is_last = i == nodes_len - 1;

            if let Ok(t) = self.analyze_node(node) {
                // Validate last expression matches function return type
                if is_last && self.current_return_type.is_some() {
                    let expected = self.current_return_type.as_ref().unwrap();
                    if !types_compatible(&t, expected) {
                        self.error(format!(
                            "TypeError: Implicit return type mismatch: expected {}, got {}",
                            expected, t
                        ));
                    }
                }
                last_type = t;
            }
        }

        self.scope_stack.pop(); // Exit scope
        Ok(last_type)
    }

    fn analyze_func_decl(&mut self, node: &mut Node) -> Result<Type, ()> {
        let (name, params, body, return_type) = match node {
            Node::FunctionDecl {
                name,
                params,
                body,
                return_type,
            } => (name, params, body, return_type),
            _ => return Err(()),
        };

        // Save previous context
        let prev_return = self.current_return_type.clone();
        let prev_function = self.current_function;

        self.current_return_type = return_type.clone();
        self.current_function = Some(*name);

        // Enter function scope and define parameters
        self.scope_stack.push();
        for param in params {
            self.scope_stack.define(Symbol {
                name: param.name,
                type_annotation: param.type_annotation.clone(),
                mutable: true,
                scope_level: self.scope_stack.current_level(),
            });
        }

        // Analyze function body
        let body_type = self.analyze_node(body)?;
        // Infer return type if not specified
        if return_type.is_none() {
            *return_type = Some(body_type.clone());
        }

        // Validate body type matches declared return type
        if let Some(expected) = return_type
            && !types_compatible(&body_type, expected)
        {
            self.error(format!(
                "TypeError: Function return type mismatch: expected {}, got {}",
                expected, body_type
            ));
        }

        self.scope_stack.pop(); // Exit function scope
        self.current_return_type = prev_return; // Restore previous context
        self.current_function = prev_function;

        Ok(Type::Void)
    }

    fn analyze_func_call(&mut self, node: &mut Node) -> Result<Type, ()> {
        let (name, args) = match node {
            Node::FunctionCall { name, args, .. } => (name, args),
            _ => return Err(()),
        };

        // Extract function name identifier
        let func_name = match name.as_ref() {
            Node::Identifier(spur) => *spur,
            _ => {
                self.error("CallError: Function name must be an identifier".to_string());
                return Err(());
            }
        };

        // Look up function signature
        let (param_types, return_type) =
            self.functions.get(&func_name).cloned().ok_or_else(|| {
                self.error(format!(
                    "NameError: Undefined function: {}",
                    self.resolve_name(&func_name)
                ));
            })?;

        // Check argument count matches parameter count
        if args.len() != param_types.len() {
            self.error(format!(
                "CallError: Function {} expects {} arguments, got {}",
                self.resolve_name(&func_name),
                param_types.len(),
                args.len()
            ));
            return Err(());
        }

        // Validate each argument type with bounds checking
        for (i, (arg, expected_type)) in args.iter_mut().zip(param_types.iter()).enumerate() {
            let arg_type = self.analyze_node(arg)?;
            if !types_compatible(&arg_type, expected_type) {
                self.error(format!(
                    "TypeError: Argument {} type mismatch for {}: expected {}, got {}",
                    i,
                    self.resolve_name(&func_name),
                    expected_type,
                    arg_type
                ));
                return Err(());
            }
        }

        // Return function's return type
        Ok(return_type.unwrap_or(Type::Void))
    }

    fn analyze_syscall(&mut self, node: &mut Node) -> Result<Type, ()> {
        let (id, _args) = match node {
            Node::SysCall { id, args, .. } => (id, args),
            _ => return Err(()),
        };

        // Return function's return type
        Ok(id.get_return_type())
    }

    fn analyze_return(&mut self, value: &mut Option<Box<Node>>) -> Result<Type, ()> {
        match value {
            Some(val) => {
                let return_type = self.analyze_node(val)?;
                // Check type matches function's expected return type
                if let Some(expected) = &self.current_return_type
                    && !types_compatible(&return_type, expected)
                {
                    self.error(format!(
                        "TypeError: Return type mismatch: expected {}, got {}",
                        expected, return_type
                    ));
                    return Err(());
                }
                Ok(return_type)
            }
            None => {
                // Void return is only valid if function expects void
                if let Some(expected) = &self.current_return_type
                    && expected != &Type::Void
                {
                    self.error(format!(
                        "TypeError: Function must return {}, but returns nothing",
                        expected
                    ));
                    return Err(());
                }
                Ok(Type::Void)
            }
        }
    }

    fn error(&mut self, message: String) {
        let context = if let Some(func) = self.current_function {
            format!("[{}]: {}", self.resolve_name(&func), message)
        } else {
            format!(" {}", message)
        };
        self.errors.push(context);
    }

    fn resolve_name(&self, spur: &Spur) -> &str {
        self.rodeo.resolve(spur)
    }
}
