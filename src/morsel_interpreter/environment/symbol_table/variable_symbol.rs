// Copyright (c) 2026 bazelik-null

use crate::morsel_interpreter::environment::types::Type;
use crate::morsel_interpreter::environment::value::Value;
use std::collections::HashMap;

/// Variable symbol with metadata
#[derive(Clone, Debug)]
pub struct VariableSymbol {
    pub name: String,
    pub type_annotation: Type,
    pub mutable: bool,
    pub scope_depth: u8,
    pub initialized: bool,
    pub value: Option<Box<Value>>,
}

impl VariableSymbol {
    pub fn new(name: String, type_annotation: Type, mutable: bool, scope_depth: u8) -> Self {
        VariableSymbol {
            name,
            type_annotation,
            mutable,
            scope_depth,
            initialized: false,
            value: None,
        }
    }

    /// Set the runtime value
    pub fn set_value(&mut self, val: Value) {
        self.value = Some(Box::new(val));
        self.initialized = true;
    }

    /// Get the runtime value
    pub fn get_value(&self) -> Option<&Value> {
        self.value.as_ref().map(|b| b.as_ref())
    }

    /// Take ownership of value (for returns)
    pub fn take_value(&mut self) -> Option<Value> {
        self.value.take().map(|b| *b)
    }
}

/// Scoped symbol table for variables
#[derive(Clone)]
pub struct VariableSymbolTable {
    // Stack of scope levels, each containing variable names at that depth
    scopes: Vec<HashMap<String, VariableSymbol>>,
}

impl VariableSymbolTable {
    pub fn new() -> Self {
        VariableSymbolTable {
            scopes: vec![HashMap::with_capacity(256)],
        }
    }

    /// Define a variable with initial value
    pub fn define_with_value(
        &mut self,
        name: String,
        type_annotation: Type,
        mutable: bool,
        value: Value,
    ) -> Result<(), String> {
        if let Some(scope) = self.scopes.last()
            && scope.contains_key(&name)
        {
            return Err(format!(
                "Variable '{}' already defined in current scope",
                name
            ));
        }

        let mut symbol = VariableSymbol::new(name.clone(), type_annotation, mutable, self.depth());
        symbol.set_value(value);

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, symbol);
        }

        Ok(())
    }

    /// Define uninitialized variable
    pub fn define(
        &mut self,
        name: String,
        type_annotation: Type,
        mutable: bool,
    ) -> Result<(), String> {
        if let Some(scope) = self.scopes.last()
            && scope.contains_key(&name)
        {
            return Err(format!(
                "Variable '{}' already defined in current scope",
                name
            ));
        }

        let symbol = VariableSymbol::new(name.clone(), type_annotation, mutable, self.depth());

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, symbol);
        }

        Ok(())
    }

    /// Get variable value by name (searches from innermost to outermost scope)
    pub fn get_value(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return symbol.get_value();
            }
        }
        None
    }

    /// Set variable value by name (searches from innermost to outermost scope)
    pub fn set_value(&mut self, name: &str, value: Value) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(symbol) = scope.get_mut(name) {
                symbol.set_value(value);
                return Ok(());
            }
        }
        Err(format!("Variable '{}' not found", name))
    }

    /// Get variable metadata (searches from innermost to outermost scope)
    pub fn lookup(&self, name: &str) -> Option<&VariableSymbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    /// Get mutable variable metadata (searches from innermost to outermost scope)
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut VariableSymbol> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(symbol) = scope.get_mut(name) {
                return Some(symbol);
            }
        }
        None
    }

    /// Check if variable exists in any scope
    pub fn exists(&self, name: &str) -> bool {
        self.scopes
            .iter()
            .rev()
            .any(|scope| scope.contains_key(name))
    }

    /// Check if variable exists in current scope only
    pub fn exists_in_current_scope(&self, name: &str) -> bool {
        self.scopes
            .last()
            .map(|scope| scope.contains_key(name))
            .unwrap_or(false)
    }

    /// Push new scope
    pub fn push_scope(&mut self) {
        if self.scopes.len() == u8::MAX as usize {
            panic!("[ERROR]: Maximum scope depth exceeded");
        }
        self.scopes.push(HashMap::with_capacity(64));
    }

    /// Pop current scope
    pub fn pop_scope(&mut self) -> Option<HashMap<String, VariableSymbol>> {
        if self.scopes.len() > 1 {
            Some(self.scopes.pop().unwrap())
        } else {
            None
        }
    }

    /// Get current scope depth
    pub fn depth(&self) -> u8 {
        (self.scopes.len() - 1) as u8
    }
}

impl Default for VariableSymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
