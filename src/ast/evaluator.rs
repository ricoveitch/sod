use super::ast::{ASTNode, BinaryExpression, VariableExpression};
use super::symbol_table::{Symbol, SymbolTable};
use crate::lexer::TokenType;

pub struct ASTEvaluator {
    symbol_table: SymbolTable,
}

impl ASTEvaluator {
    pub fn new() -> ASTEvaluator {
        ASTEvaluator {
            symbol_table: SymbolTable::new(),
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

    fn eval_node(&mut self, node: ASTNode) -> Option<Symbol> {
        match node {
            ASTNode::Number(value) => Some(Symbol::Number(value)),
            ASTNode::BinaryExpression(be) => self.eval_binary_expression(be),
            ASTNode::UnaryExpression(n) => self.eval_unary_expression(*n),
            ASTNode::VariableExpression(ve) => {
                self.eval_variable_statement(ve);
                None
            }
            ASTNode::Variable(name) => Some(self.symbol_table.get(&name).clone()),
            ASTNode::FunctionExpression(fe) => {
                self.symbol_table
                    .insert(&fe.name, Symbol::Function(fe.body));
                None
            }
            ASTNode::FunctionCall(name) => self.eval_function(name),
            _ => None,
        }
    }

    fn eval_function(&mut self, name: String) -> Option<Symbol> {
        let body = match self.symbol_table.get(&name) {
            Symbol::Function(f) => f.clone(),
            _ => return None,
        };

        self.symbol_table.push_scope(&name);

        for line in *body {
            match line {
                ASTNode::ReturnExpression(expr) => {
                    let res = self.eval_node(*expr);
                    self.symbol_table.pop_scope();
                    return res;
                }
                _ => self.eval_node(line),
            };
        }

        self.symbol_table.pop_scope();
        None
    }

    fn eval_variable_statement(&mut self, node: VariableExpression) {
        if let Some(val) = self.eval_node(*node.value) {
            self.symbol_table.insert(&node.name, val);
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
