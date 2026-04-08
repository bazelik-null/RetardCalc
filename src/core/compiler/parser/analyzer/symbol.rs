use crate::core::shared::types::Type;
use lasso::Spur;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: Spur,
    pub type_annotation: Type,
    pub mutable: bool,
}

#[derive(Clone)]
pub struct Scope {
    pub symbols: HashMap<Spur, Symbol>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn define(&mut self, symbol: Symbol) {
        self.symbols.insert(symbol.name, symbol);
    }

    pub fn lookup_local(&self, name: Spur) -> Option<Symbol> {
        self.symbols.get(&name).cloned()
    }
}

pub struct ScopeStack {
    scopes: Vec<Scope>,
}

impl ScopeStack {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn pop(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn define(&mut self, symbol: Symbol) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.define(symbol);
        }
    }

    pub fn lookup(&self, name: Spur) -> Option<Symbol> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.lookup_local(name) {
                return Some(symbol);
            }
        }
        None
    }
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}
