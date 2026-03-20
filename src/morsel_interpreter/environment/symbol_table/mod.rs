// Copyright (c) 2026 bazelik-null

pub mod function_symbol;
pub mod variable_symbol;

// Symbol table

use crate::morsel_interpreter::environment::symbol_table::function_symbol::FunctionSymbolTable;
use crate::morsel_interpreter::environment::symbol_table::variable_symbol::{
    VariableSymbol, VariableSymbolTable,
};
use std::collections::HashMap;
use std::fmt;

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
    pub fn pop_scope(&mut self) -> Option<HashMap<String, VariableSymbol>> {
        self.variables.pop_scope()
    }

    /// Get current scope depth
    pub fn depth(&self) -> usize {
        self.variables.depth()
    }

    /// Get a report of all symbols
    pub fn report(&self) -> String {
        let mut report = String::new();
        report.push_str("┌─ SYMBOL TABLE REPORT ─┐\n\n");

        // Functions section
        if !self.functions.all_functions().is_empty() {
            report.push_str("├─ FUNCTIONS\n");

            for func in self.functions.all_functions() {
                if func.is_builtin() {
                    continue;
                }

                let params = func
                    .parameters
                    .iter()
                    .map(|p| format!("{}: {}", p.name, p.type_annotation))
                    .collect::<Vec<_>>()
                    .join(", ");

                let variadic = if func.is_variadic { ", ..." } else { "" };
                report.push_str(&format!(
                    "│  └─ {} ({}{}) -> {}\n",
                    func.name, params, variadic, func.return_type
                ));

                report.push_str(&format!("│     Scope depth: {}\n", func.scope_depth));
            }
            report.push_str("│\n");
        }

        // Variables section
        let all_vars = self.variables.all_vars();
        if !all_vars.is_empty() {
            report.push_str("├─ VARIABLES\n");

            for var in all_vars {
                let mutability = if var.mutable { "mutable" } else { "immutable" };
                report.push_str(&format!(
                    "│  └─ {}: {} [{}] (depth: {})\n",
                    var.name, var.type_annotation, mutability, var.scope_depth
                ));
            }
            report.push_str("│\n");
        }

        report.push_str("└─────────────────────┘\n");
        report
    }
}

impl fmt::Display for SymbolTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.report())
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
