// Copyright (c) 2026 bazelik-null

use crate::morsel_interpreter::environment::morsel_std::BuiltinFunctionDispatcher;
use crate::morsel_interpreter::environment::types::Type;
use crate::morsel_interpreter::parser::ast_node::Node;
use std::collections::HashMap;

const BUILTIN_NAMESPACES: &[&str] = &["std", "std::math", "std::io"];

/// Represents a function parameter with RTTI
#[derive(Clone, Debug)]
pub struct FunctionParamSymbol {
    pub name: String,
    pub type_annotation: Type,
}

impl FunctionParamSymbol {
    pub fn new(name: String, param_type: Type) -> Self {
        FunctionParamSymbol {
            name,
            type_annotation: param_type,
        }
    }
}

/// Represents a function symbol with RTTI
#[derive(Clone, Debug)]
pub struct FunctionSymbol {
    pub name: String,
    pub namespace: String,
    pub parameters: Vec<FunctionParamSymbol>,
    pub return_type: Type,
    pub scope_depth: usize,
    pub implementation: Option<Box<Node>>, // None for builtins
    pub is_variadic: bool,                 // True if function accepts infinite parameters
}

impl FunctionSymbol {
    pub fn new(
        name: String,
        namespace: String,
        parameters: Vec<FunctionParamSymbol>,
        return_type: Type,
        scope_depth: usize,
        implementation: Option<Box<Node>>,
    ) -> Self {
        FunctionSymbol {
            name,
            namespace,
            parameters,
            return_type,
            scope_depth,
            implementation,
            is_variadic: false,
        }
    }

    pub fn builtin(
        name: String,
        namespace: String,
        parameters: Vec<FunctionParamSymbol>,
        return_type: Type,
    ) -> Self {
        FunctionSymbol {
            name,
            namespace,
            parameters,
            return_type,
            scope_depth: 0,
            implementation: None,
            is_variadic: false,
        }
    }

    pub fn builtin_variadic(
        name: String,
        namespace: String,
        parameters: Vec<FunctionParamSymbol>,
        return_type: Type,
    ) -> Self {
        FunctionSymbol {
            name,
            namespace,
            parameters,
            return_type,
            scope_depth: 0,
            implementation: None,
            is_variadic: true,
        }
    }

    /// Get the fully qualified name (namespace::name)
    pub fn fully_qualified_name(&self) -> String {
        if self.namespace.is_empty() {
            self.name.clone()
        } else {
            format!("{}::{}", self.namespace, self.name)
        }
    }

    /// Check if this function is a builtin (based on namespace)
    pub fn is_builtin(&self) -> bool {
        BUILTIN_NAMESPACES
            .iter()
            .any(|&ns| self.namespace == ns || self.namespace.starts_with(&format!("{}::", ns)))
    }

    /// Get the number of parameters
    pub fn param_count(&self) -> usize {
        self.parameters.len()
    }

    /// Get parameter type by index
    pub fn get_param_type(&self, index: usize) -> Option<Type> {
        self.parameters.get(index).map(|p| p.type_annotation)
    }

    /// Get parameter type by name
    pub fn get_param_type_by_name(&self, name: &str) -> Option<Type> {
        self.parameters
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.type_annotation)
    }
}

/// Global symbol table for functions with namespace support
#[derive(Clone)]
pub struct FunctionSymbolTable {
    // Key: fully qualified name (namespace::name)
    functions: HashMap<String, FunctionSymbol>,
}

impl FunctionSymbolTable {
    pub fn new() -> Self {
        let mut table = FunctionSymbolTable {
            functions: HashMap::new(),
        };

        BuiltinFunctionDispatcher::register_builtins(&mut table);

        table
    }

    /// Define a function with namespace support
    pub fn define(&mut self, symbol: FunctionSymbol) -> Result<(), String> {
        let fq_name = symbol.fully_qualified_name();

        if let Some(existing) = self.functions.get(&fq_name) {
            if existing.is_builtin() {
                return Err(format!("Cannot override builtin function '{}'", fq_name));
            }
            return Err(format!(
                "Function '{}' is already defined at scope depth {}",
                fq_name, existing.scope_depth
            ));
        }

        self.functions.insert(fq_name, symbol);
        Ok(())
    }

    /// Lookup a function by fully qualified name
    pub fn lookup(&self, fully_qualified_name: &str) -> Option<&FunctionSymbol> {
        self.functions.get(fully_qualified_name)
    }

    /// Lookup a function by name with optional namespace prefix
    pub fn lookup_with_namespace(
        &self,
        name: &str,
        namespace: Option<&str>,
    ) -> Option<&FunctionSymbol> {
        if let Some(ns) = namespace {
            let fq_name = format!("{}::{}", ns, name);
            self.functions.get(&fq_name)
        } else {
            // Try exact match first
            if let Some(func) = self.functions.get(name) {
                return Some(func);
            }
            // Try to find in builtin namespaces
            for &builtin_ns in BUILTIN_NAMESPACES {
                let fq_name = format!("{}::{}", builtin_ns, name);
                if let Some(func) = self.functions.get(&fq_name) {
                    return Some(func);
                }
            }
            None
        }
    }

    /// Lookup mutable reference
    pub fn lookup_mut(&mut self, fully_qualified_name: &str) -> Option<&mut FunctionSymbol> {
        self.functions.get_mut(fully_qualified_name)
    }

    /// Check if function exists
    pub fn exists(&self, fully_qualified_name: &str) -> bool {
        self.functions.contains_key(fully_qualified_name)
    }

    /// Get all function names (fully qualified)
    pub fn function_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    /// Get all functions
    pub fn all_functions(&self) -> Vec<&FunctionSymbol> {
        self.functions.values().collect()
    }

    /// Get functions in a specific namespace
    pub fn functions_in_namespace(&self, namespace: &str) -> Vec<&FunctionSymbol> {
        self.functions
            .values()
            .filter(|func| func.namespace == namespace)
            .collect()
    }

    /// Remove a function
    pub fn remove(&mut self, fully_qualified_name: &str) -> Option<FunctionSymbol> {
        self.functions.remove(fully_qualified_name)
    }

    /// Clear all user-defined functions (keeps builtins)
    pub fn clear_user_functions(&mut self) {
        self.functions.retain(|_, func| func.is_builtin());
    }

    /// Get function parameter count
    pub fn get_param_count(&self, fully_qualified_name: &str) -> Option<usize> {
        self.functions
            .get(fully_qualified_name)
            .map(|f| f.param_count())
    }

    /// Validate function arguments count
    pub fn validate_arg_count(
        &self,
        fully_qualified_name: &str,
        arg_count: usize,
    ) -> Result<(), String> {
        match self.lookup(fully_qualified_name) {
            Some(func) => {
                if func.is_variadic {
                    // For variadic functions, check minimum required args
                    if arg_count < func.param_count() {
                        return Err(format!(
                            "Function '{}' expects at least {} argument(s), got {}",
                            fully_qualified_name,
                            func.param_count(),
                            arg_count
                        ));
                    }
                    Ok(())
                } else {
                    // For fixed functions, exact match required
                    if arg_count != func.param_count() {
                        Err(format!(
                            "Function '{}' expects {} argument(s), got {}",
                            fully_qualified_name,
                            func.param_count(),
                            arg_count
                        ))
                    } else {
                        Ok(())
                    }
                }
            }
            None => Err(format!("Function '{}' not found", fully_qualified_name)),
        }
    }

    /// Validate parameter types
    pub fn validate_param_types(
        &self,
        fully_qualified_name: &str,
        arg_types: &[Type],
    ) -> Result<(), String> {
        match self.lookup(fully_qualified_name) {
            Some(func) => {
                if func.is_variadic {
                    // For variadic functions, check minimum required args
                    if arg_types.len() < func.param_count() {
                        return Err(format!(
                            "Function '{}' expects at least {} argument(s), got {}",
                            fully_qualified_name,
                            func.param_count(),
                            arg_types.len()
                        ));
                    }

                    // Validate types for all arguments against the first parameter type
                    if let Some(first_param_type) = func.get_param_type(0) {
                        for (i, arg_type) in arg_types.iter().enumerate() {
                            if !arg_type.is_compatible_with(&first_param_type) {
                                return Err(format!(
                                    "Function '{}' argument {} expects type {}, got {}",
                                    fully_qualified_name, i, first_param_type, arg_type
                                ));
                            }
                        }
                    }
                    Ok(())
                } else {
                    // For fixed functions, exact match required
                    if arg_types.len() != func.param_count() {
                        return Err(format!(
                            "Function '{}' expects {} argument(s), got {}",
                            fully_qualified_name,
                            func.param_count(),
                            arg_types.len()
                        ));
                    }

                    for (i, arg_type) in arg_types.iter().enumerate() {
                        if let Some(param_type) = func.get_param_type(i)
                            && !arg_type.is_compatible_with(&param_type)
                        {
                            return Err(format!(
                                "Function '{}' parameter {} expects type {}, got {}",
                                fully_qualified_name, i, param_type, arg_type
                            ));
                        }
                    }
                    Ok(())
                }
            }
            None => Err(format!("Function '{}' not found", fully_qualified_name)),
        }
    }

    /// Check if function is variadic
    pub fn is_variadic(&self, fully_qualified_name: &str) -> bool {
        self.lookup(fully_qualified_name)
            .map(|func| func.is_variadic)
            .unwrap_or(false)
    }
}

impl Default for FunctionSymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
