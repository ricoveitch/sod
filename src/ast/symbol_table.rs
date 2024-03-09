use std::collections::HashMap;

use super::symbol::Symbol;

type Scope = String;
pub struct SymbolTable {
    scoped_table: HashMap<Scope, HashMap<String, Symbol>>,
    scope: Vec<Scope>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        let global_scope = "global";
        let mut scoped_table = HashMap::new();
        scoped_table.insert(global_scope.to_string(), HashMap::new());

        SymbolTable {
            scoped_table,
            scope: vec![global_scope.to_string()],
        }
    }

    fn curr_scope(&self) -> &String {
        self.scope.last().unwrap()
    }

    pub fn get(&self, name: &str) -> &Symbol {
        for scope in self.scope.iter().rev() {
            if let Some(symbol) = self
                .scoped_table
                .get(scope)
                .and_then(|symbol_table| symbol_table.get(name))
            {
                return symbol;
            }
        }

        panic!("unknown symbol {}", name)
    }

    pub fn insert(&mut self, name: &str, symbol: Symbol) {
        let scope = self.curr_scope().to_owned();
        match self.scoped_table.get_mut(&scope) {
            Some(symbol_table) => symbol_table.insert(name.to_string(), symbol),
            None => panic!("scope {} not found", scope),
        };
    }

    pub fn push_scope(&mut self, scope: &str) {
        self.scope.push(scope.to_string());
        self.scoped_table.insert(scope.to_string(), HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        match self.scope.pop() {
            Some(scope) => self.scoped_table.remove(&scope),
            None => panic!("exiting scope was not found"),
        };
    }
}
