// Copyright (c) 2026 bazelik-null

use crate::morsel_interpreter::environment::morsel_std::BuiltinFunctionDispatcher;
use crate::morsel_interpreter::environment::types::Type;
use crate::morsel_interpreter::parser::ast_node::Node;
use std::collections::HashMap;
use std::sync::Arc;

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

/// Represents a function metadata
#[derive(Clone, Debug)]
pub struct FunctionMetadata {
    pub name: String,
    pub namespace: String,
    pub fully_qualified_name: String,
    pub param_count: usize,
    pub parameters: Vec<FunctionParamSymbol>,
    pub return_type: Type,
    pub is_builtin: bool,
    pub is_variadic: bool,
}

impl FunctionMetadata {
    pub fn new(
        name: String,
        namespace: String,
        parameters: Vec<FunctionParamSymbol>,
        return_type: Type,
        is_builtin: bool,
        is_variadic: bool,
    ) -> Self {
        let param_count = parameters.len();
        let fully_qualified_name = if namespace.is_empty() {
            name.clone()
        } else {
            format!("{}::{}", namespace, name)
        };

        FunctionMetadata {
            name,
            namespace,
            fully_qualified_name,
            param_count,
            parameters,
            return_type,
            is_builtin,
            is_variadic,
        }
    }

    pub fn get_param_type(&self, index: usize) -> Option<Type> {
        self.parameters.get(index).map(|p| p.type_annotation)
    }

    pub fn get_param_type_by_name(&self, name: &str) -> Option<Type> {
        self.parameters
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.type_annotation)
    }
}

/// Represents a function symbol with RTTI
#[derive(Clone, Debug)]
pub struct FunctionSymbol {
    pub metadata: Arc<FunctionMetadata>,
    pub scope_depth: u8,
    pub implementation: Option<Box<Node>>,
}

impl FunctionSymbol {
    pub fn new(
        name: String,
        namespace: String,
        parameters: Vec<FunctionParamSymbol>,
        return_type: Type,
        scope_depth: u8,
        implementation: Option<Box<Node>>,
        is_variadic: bool,
    ) -> Self {
        let is_builtin = Self::check_builtin(&namespace);
        FunctionSymbol {
            metadata: Arc::new(FunctionMetadata::new(
                name,
                namespace,
                parameters,
                return_type,
                is_builtin,
                is_variadic,
            )),
            scope_depth,
            implementation,
        }
    }

    fn check_builtin(namespace: &str) -> bool {
        BUILTIN_NAMESPACES
            .iter()
            .any(|&ns| namespace == ns || namespace.starts_with(&format!("{}::", ns)))
    }
}

/// Global symbol table for functions
#[derive(Clone)]
pub struct FunctionSymbolTable {
    functions: HashMap<String, FunctionSymbol>,
    name_index: HashMap<String, Vec<String>>,
}

impl FunctionSymbolTable {
    pub fn new() -> Self {
        let mut table = FunctionSymbolTable {
            functions: HashMap::new(),
            name_index: HashMap::new(),
        };

        BuiltinFunctionDispatcher::register_builtins(&mut table);

        table
    }

    /// Define a function
    pub fn define(&mut self, symbol: FunctionSymbol) -> Result<(), String> {
        let fq_name = &symbol.metadata.fully_qualified_name;
        let name = &symbol.metadata.name;

        if let Some(existing) = self.functions.get(fq_name) {
            if existing.metadata.is_builtin {
                return Err(format!("Cannot override builtin function '{}'", fq_name));
            }
            return Err(format!(
                "Function '{}' is already defined at scope depth {}",
                fq_name, existing.scope_depth
            ));
        }

        self.name_index
            .entry(name.clone())
            .or_default()
            .push(fq_name.clone());

        self.functions.insert(fq_name.clone(), symbol);
        Ok(())
    }

    /// Define multiple functions at once
    pub fn define_batch(&mut self, symbols: Vec<FunctionSymbol>) -> Result<(), String> {
        for symbol in symbols {
            self.define(symbol)?;
        }
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
            return self.functions.get(&fq_name);
        }

        if let Some(func) = self.functions.get(name) {
            return Some(func);
        }

        if let Some(candidates) = self.name_index.get(name) {
            let mut first_candidate = None;
            for candidate_fq_name in candidates {
                let func = &self.functions[candidate_fq_name];

                if func.metadata.is_builtin {
                    return Some(func);
                }

                if first_candidate.is_none() {
                    first_candidate = Some(func);
                }
            }
            return first_candidate;
        }

        None
    }

    /// Lookup mutable reference
    pub fn lookup_mut(&mut self, fully_qualified_name: &str) -> Option<&mut FunctionSymbol> {
        self.functions.get_mut(fully_qualified_name)
    }

    /// Remove a function
    pub fn remove(&mut self, fully_qualified_name: &str) -> Option<FunctionSymbol> {
        if let Some(symbol) = self.functions.remove(fully_qualified_name) {
            let name = &symbol.metadata.name;
            if let Some(candidates) = self.name_index.get_mut(name) {
                candidates.retain(|fq| fq != fully_qualified_name);
                if candidates.is_empty() {
                    self.name_index.remove(name);
                }
            }
            Some(symbol)
        } else {
            None
        }
    }

    /// Clear all user-defined functions (keeps builtins)
    pub fn clear_user_functions(&mut self) {
        self.functions.retain(|_, func| func.metadata.is_builtin);
        self.rebuild_name_index();
    }

    /// Rebuild the name index
    fn rebuild_name_index(&mut self) {
        self.name_index.clear();
        for (fq_name, symbol) in &self.functions {
            let name = &symbol.metadata.name;
            self.name_index
                .entry(name.clone())
                .or_default()
                .push(fq_name.clone());
        }
    }

    /// Validate parameter types and count
    pub fn validate_call(
        &self,
        fully_qualified_name: &str,
        arg_types: &[Type],
    ) -> Result<(), String> {
        match self.lookup(fully_qualified_name) {
            Some(func) => {
                let metadata = &func.metadata;

                if metadata.is_variadic {
                    if arg_types.len() < metadata.param_count {
                        return Err(format!(
                            "Function '{}' expects at least {} argument(s), got {}",
                            fully_qualified_name,
                            metadata.param_count,
                            arg_types.len()
                        ));
                    }

                    if let Some(first_param_type) = metadata.parameters.first() {
                        for (i, arg_type) in arg_types.iter().enumerate() {
                            if !arg_type.is_compatible_with(&first_param_type.type_annotation) {
                                return Err(format!(
                                    "Function '{}' argument {} expects type {}, got {}",
                                    fully_qualified_name,
                                    i,
                                    first_param_type.type_annotation,
                                    arg_type
                                ));
                            }
                        }
                    }
                    Ok(())
                } else {
                    if arg_types.len() != metadata.param_count {
                        return Err(format!(
                            "Function '{}' expects {} argument(s), got {}",
                            fully_qualified_name,
                            metadata.param_count,
                            arg_types.len()
                        ));
                    }

                    for (i, arg_type) in arg_types.iter().enumerate() {
                        if let Some(param) = metadata.parameters.get(i)
                            && !arg_type.is_compatible_with(&param.type_annotation)
                        {
                            return Err(format!(
                                "Function '{}' parameter {} expects type {}, got {}",
                                fully_qualified_name, i, param.type_annotation, arg_type
                            ));
                        }
                    }
                    Ok(())
                }
            }
            None => Err(format!("Function '{}' not found", fully_qualified_name)),
        }
    }
}

impl Default for FunctionSymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
