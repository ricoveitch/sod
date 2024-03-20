use std::collections::HashMap;

use super::{
    scope::{ScopeKind, ScopeStack},
    symbol::Symbol,
};

type ScopeName = String;
type SymbolName = String;

pub struct SymbolTable {
    scoped_table: HashMap<ScopeName, HashMap<SymbolName, Symbol>>,
    scope: ScopeStack,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        let mut scoped_table = HashMap::new();
        scoped_table.insert("global".to_string(), HashMap::new());

        SymbolTable {
            scoped_table,
            scope: ScopeStack::new(),
        }
    }

    fn find(&self, symbol_name: &str) -> Option<(&String, &Symbol)> {
        for scope in self.scope.curr_stack().iter().rev() {
            if let Some(symbol) = self
                .scoped_table
                .get(&scope.name)
                .and_then(|symbol_table| symbol_table.get(symbol_name))
            {
                return Some((&scope.name, symbol));
            }
        }

        None
    }

    pub fn get(&self, symbol_name: &str) -> Option<&Symbol> {
        if let Some((_, symbol)) = self.find(symbol_name) {
            return Some(symbol);
        }

        None
    }

    fn get_mut(&mut self, symbol_name: &str) -> Option<&mut Symbol> {
        let scope_name = match self.find(symbol_name) {
            Some((s, _)) => s.clone(),
            None => return None,
        };

        let symbol = self
            .scoped_table
            .get_mut(&scope_name)
            .and_then(|symbol_table| symbol_table.get_mut(symbol_name))
            .unwrap();

        Some(symbol)
    }

    pub fn insert(&mut self, name: &str, symbol: Symbol) {
        if let Some(existing_symbol) = self.get_mut(name) {
            *existing_symbol = symbol;
            return;
        }

        let curr_scope_name = self.scope.curr().name.clone();
        match self.scoped_table.get_mut(&curr_scope_name) {
            Some(symbol_table) => symbol_table.insert(name.to_string(), symbol),
            None => panic!("scope {} not found", curr_scope_name),
        };
    }

    pub fn push_scope(&mut self, kind: ScopeKind) {
        let scope_name = self.scope.push(kind);
        self.scoped_table.insert(scope_name, HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        let popped_scope = self.scope.pop();
        self.scoped_table.remove(&popped_scope.name);
    }
}
