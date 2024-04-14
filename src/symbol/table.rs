use std::collections::HashMap;

use super::{
    scope::{ScopeKind, ScopeStack, GLOBAL_SCOPE_ID},
    symbol::Symbol,
};

type ScopeID = usize;
type SymbolName = String;

pub struct SymbolTable {
    pub scoped_table: HashMap<ScopeID, HashMap<SymbolName, Symbol>>,
    scope: ScopeStack,
}

impl SymbolTable {
    pub fn from(global_vars: Vec<(&str, Symbol)>) -> Self {
        let mut scoped_table = HashMap::new();
        scoped_table.insert(GLOBAL_SCOPE_ID, HashMap::new());

        let mut symbol_table = SymbolTable {
            scoped_table,
            scope: ScopeStack::new(),
        };

        for (key, value) in global_vars {
            symbol_table.set(key, value);
        }

        symbol_table
    }

    fn find(&self, symbol_name: &str) -> Option<(ScopeID, &Symbol)> {
        for scope in self.scope.curr_stack().iter().rev() {
            if let Some(symbol) = self
                .scoped_table
                .get(&scope.id)
                .and_then(|symbol_table| symbol_table.get(symbol_name))
            {
                return Some((scope.id, symbol));
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

    pub fn get_mut(&mut self, symbol_name: &str) -> Option<&mut Symbol> {
        let scope_id = match self.find(symbol_name) {
            Some((id, _)) => id,
            None => return None,
        };

        let symbol = self
            .scoped_table
            .get_mut(&scope_id)
            .and_then(|symbol_table| symbol_table.get_mut(symbol_name))
            .unwrap();

        Some(symbol)
    }

    pub fn set(&mut self, name: &str, symbol: Symbol) {
        if let Some(existing_symbol) = self.get_mut(name) {
            *existing_symbol = symbol;
            return;
        }

        let curr_scope_id = self.scope.curr().id;
        match self.scoped_table.get_mut(&curr_scope_id) {
            Some(symbol_table) => symbol_table.insert(name.to_string(), symbol),
            None => panic!("scope {} not found", curr_scope_id),
        };
    }

    pub fn push_scope(&mut self, kind: ScopeKind) {
        let scope_id = self.scope.push(kind);
        self.scoped_table.insert(scope_id, HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        let popped_scope = self.scope.pop();
        self.scoped_table.remove(&popped_scope.id);
    }
}
