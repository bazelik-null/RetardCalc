// Copyright (c) 2026 bazelik-null

pub mod function_symbol;
pub mod variable_symbol;

// Symbol table

use crate::morsel_interpreter::environment::symbol_table::function_symbol::FunctionSymbolTable;
use crate::morsel_interpreter::environment::symbol_table::variable_symbol::VariableSymbolTable;

/// Complete symbol table combining variables and functions
#[derive(Clone)]
pub struct SymbolTable {
    pub variables: VariableSymbolTable,
    pub functions: FunctionSymbolTable,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            variables: VariableSymbolTable::new(),
            functions: FunctionSymbolTable::new(),
        }
    }

    /// Push a new variable scope
    pub fn push_scope(&mut self) {
        self.variables.push_scope();
    }

    /// Pop the current variable scope
    pub fn pop_scope(&mut self) {
        self.variables.pop_scope();
    }

    /// Get current scope depth
    pub fn depth(&self) -> u8 {
        self.variables.depth()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
