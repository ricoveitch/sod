use std::collections::HashMap;

use super::ast::{ASTNode, BinaryExpression, VariableExpression};
use crate::lexer::TokenType;

#[derive(Debug, Clone)]
pub enum Symbol {
    Number(f64),
    Function(Box<Vec<ASTNode>>),
}

pub struct ASTEvaluator {
    symbol_table: HashMap<String, Symbol>,
    scope: Vec<String>,
}

impl ASTEvaluator {
    pub fn new() -> ASTEvaluator {
        ASTEvaluator {
            symbol_table: HashMap::new(),
            scope: vec![],
        }
    }

    pub fn eval(&mut self, program: ASTNode) -> Vec<Option<Symbol>> {
        let mut result = vec![];
        match program {
            ASTNode::Program(root) => {
                for line in *root {
                    result.push(self.eval_node(line));
                }
                result
            }
            _ => result,
        }
    }

    fn get_symbol_key(&self, symbol_name: &str, depth: usize) -> String {
        let scope_path = self.scope[0..depth].join(".");
        format!("{}.{}", scope_path, symbol_name)
    }

    fn get_symbol(&self, name: &str) -> &Symbol {
        for depth in (0..=self.scope.len()).rev() {
            let key = self.get_symbol_key(name, depth);
            match self.symbol_table.get(&key) {
                Some(symbol) => return symbol,
                None => (),
            }
        }

        panic!("unknown identifier {}", name);
    }

    fn insert_symbol(&mut self, name: &str, symbol: Symbol) {
        let key = self.get_symbol_key(name, self.scope.len());
        self.symbol_table.insert(key, symbol);
    }

    fn eval_node(&mut self, node: ASTNode) -> Option<Symbol> {
        match node {
            ASTNode::Number(value) => Some(Symbol::Number(value)),
            ASTNode::BinaryExpression(be) => self.eval_binary_expression(be),
            ASTNode::UnaryExpression(n) => self.eval_unary_expression(*n),
            ASTNode::VariableExpression(ve) => {
                self.eval_variable_statement(ve);
                None
            }
            ASTNode::Variable(name) => Some(self.get_symbol(&name).clone()),
            ASTNode::FunctionExpression(fe) => {
                self.insert_symbol(&fe.name, Symbol::Function(fe.body));
                None
            }
            ASTNode::FunctionCall(name) => self.eval_function(name),
            _ => None,
        }
    }

    fn eval_function(&mut self, name: String) -> Option<Symbol> {
        let body = match self.get_symbol(&name) {
            Symbol::Function(f) => f.clone(),
            _ => return None,
        };

        self.scope.push(name);

        for line in *body {
            match line {
                ASTNode::ReturnExpression(expr) => {
                    let res = self.eval_node(*expr);
                    self.scope.pop();
                    return res;
                }
                _ => self.eval_node(line),
            };
        }

        self.scope.pop();
        None
    }

    fn eval_variable_statement(&mut self, node: VariableExpression) {
        if let Some(val) = self.eval_node(*node.value) {
            self.insert_symbol(&node.name, val);
        }
    }

    fn eval_unary_expression(&mut self, node: ASTNode) -> Option<Symbol> {
        let symbol = match self.eval_node(node) {
            Some(s) => s,
            None => return None,
        };

        match symbol {
            Symbol::Number(num) => Some(Symbol::Number(-num)),
            _ => None,
        }
    }

    fn eval_binary_expression(&mut self, be: BinaryExpression) -> Option<Symbol> {
        let l = match self.eval_node(*be.left) {
            Some(v) => match v {
                Symbol::Number(n) => n,
                _ => return None,
            },
            None => return None,
        };
        let r = match self.eval_node(*be.right) {
            Some(v) => match v {
                Symbol::Number(n) => n,
                _ => return None,
            },
            None => return None,
        };

        let val = match be.operator {
            TokenType::Plus => l + r,
            TokenType::Minus => l - r,
            TokenType::Asterisk => l * r,
            TokenType::ForwardSlash => l / r,
            TokenType::Carat => l.powf(r),
            _ => panic!(""),
        };

        Some(Symbol::Number(val))
    }
}
