// Copyright (c) 2026 bazelik-null

use crate::morsel_core::parsing::node::Node;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub enum ReturnType {
    Value(f64),

    #[default]
    Void,
}

pub struct FunctionTable {
    functions: HashMap<String, FunctionInfo>,
}

/// Metadata about each function
#[derive(Clone, Debug)]
pub struct FunctionInfo {
    pub name: String,
    pub builtin: bool,

    pub min_args: usize,
    pub max_args: Option<usize>, // None = unlimited

    pub implementation: Option<Node>, // None if builtin
}

impl Default for FunctionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionTable {
    /// Create a new function table with all supported functions
    pub fn new() -> Self {
        let builtin = init_builtins();

        FunctionTable { functions: builtin }
    }

    /// Register a new user-defined function
    pub fn register_function(&mut self, info: FunctionInfo) -> Result<(), String> {
        let key = info.name.to_lowercase();

        if self.functions.contains_key(&key) {
            if self.functions[&key].builtin {
                return Err(format!("Cannot override builtin function '{}'", info.name));
            }
            return Err(format!("Function '{}' is already registered", info.name));
        }

        self.functions.insert(key, info);
        Ok(())
    }

    /// Update an existing function (overwrite)
    pub fn update_function(&mut self, info: FunctionInfo) -> Result<(), String> {
        let key = info.name.to_lowercase();

        if !self.functions.contains_key(&key) {
            return Err(format!("Function '{}' does not exist", info.name));
        }

        self.functions.insert(key, info);
        Ok(())
    }

    /// Remove a function from the table
    pub fn remove_function(&mut self, name: &str) -> Result<FunctionInfo, String> {
        let key = name.to_lowercase();

        self.functions
            .remove(&key)
            .ok_or_else(|| format!("Function '{}' not found", name))
    }

    /// Get mutable reference to function info (for direct modification)
    pub fn get_function_mut(&mut self, name: &str) -> Option<&mut FunctionInfo> {
        self.functions.get_mut(&name.to_lowercase())
    }

    /// Clear all user-defined functions (keeps builtins)
    pub fn clear_user_functions(&mut self) {
        self.functions.retain(|_, info| info.builtin);
    }

    /// Check if a function exists
    pub fn is_function(&self, name: &str) -> bool {
        self.functions.contains_key(&name.to_lowercase())
    }

    /// Check if a function is builtin
    pub fn is_builtin(&self, name: &str) -> bool {
        if !self.is_function(name) {
            return false;
        }

        self.functions.get(&name.to_lowercase()).unwrap().builtin
    }

    /// Get function info
    pub fn get_function(&self, name: &str) -> Option<&FunctionInfo> {
        self.functions.get(&name.to_lowercase())
    }

    /// Clone function info
    pub fn get_function_owned(&self, name: &str) -> Option<FunctionInfo> {
        self.functions.get(&name.to_lowercase()).cloned()
    }

    /// Get all function names
    pub fn function_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    /// Validate argument count
    pub fn validate_args(&self, name: &str, arg_count: usize) -> Result<(), String> {
        match self.get_function(name) {
            Some(info) => {
                if arg_count < info.min_args {
                    return Err(format!(
                        "Function '{}' requires at least {} argument(s), got {}",
                        name, info.min_args, arg_count
                    ));
                }
                if let Some(max) = info.max_args
                    && arg_count > max
                {
                    return Err(format!(
                        "Function '{}' accepts at most {} argument(s), got {}",
                        name, max, arg_count
                    ));
                }
                Ok(())
            }
            None => Err(format!("Unknown function: '{}'", name)),
        }
    }
}

fn init_builtins() -> HashMap<String, FunctionInfo> {
    [
        // Single-argument functions
        ("sqrt", 1, Some(1)),
        ("cbrt", 1, Some(1)),
        ("ln", 1, Some(1)),
        ("sin", 1, Some(1)),
        ("cos", 1, Some(1)),
        ("tan", 1, Some(1)),
        ("asin", 1, Some(1)),
        ("acos", 1, Some(1)),
        ("atan", 1, Some(1)),
        ("abs", 1, Some(1)),
        ("round", 1, Some(1)),
        ("floor", 1, Some(1)),
        ("ceil", 1, Some(1)),
        // Multi-argument functions
        ("root", 2, Some(2)),
        ("log", 2, Some(2)),
        ("min", 1, None),
        ("max", 1, None),
        // I/O functions
        ("print", 1, None),
    ]
    .into_iter()
    .map(|(name, min, max)| {
        (
            name.to_string(),
            FunctionInfo {
                name: name.to_string(),
                builtin: true,
                min_args: min,
                max_args: max,
                implementation: None,
            },
        )
    })
    .collect()
}
