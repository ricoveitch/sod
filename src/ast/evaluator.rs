use std::collections::HashMap;

use super::ast::{ASTNode, BinaryExpression, VariableExpression};
use crate::lexer::TokenType;

pub struct ASTEvaluator {
    symbol_table: HashMap<String, f64>,
}

impl ASTEvaluator {
    pub fn new() -> ASTEvaluator {
        ASTEvaluator {
            symbol_table: HashMap::new(),
        }
    }

    pub fn eval(&mut self, program: ASTNode) -> Vec<Option<f64>> {
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

    fn eval_node(&mut self, node: ASTNode) -> Option<f64> {
        match node {
            ASTNode::Number(value) => return Some(value),
            ASTNode::BinaryExpression(be) => self.eval_binary_expression(be),
            ASTNode::UnaryExpression(n) => self.eval_unary_expression(*n),
            ASTNode::VariableExpression(ve) => {
                self.eval_variable_statement(ve);
                return None;
            }
            ASTNode::Variable(name) => match self.symbol_table.get(&name) {
                Some(val) => Some(val.to_owned()),
                None => panic!("unknown variable {}", name),
            },
            _ => None,
        }
    }

    fn eval_variable_statement(&mut self, node: VariableExpression) {
        if let Some(val) = self.eval_node(*node.value) {
            self.symbol_table.insert(node.name, val);
        }
    }

    fn eval_unary_expression(&mut self, node: ASTNode) -> Option<f64> {
        self.eval_node(node).map(|v| -v)
    }

    fn eval_binary_expression(&mut self, be: BinaryExpression) -> Option<f64> {
        let l = match self.eval_node(*be.left) {
            Some(v) => v,
            None => return None,
        };
        let r = match self.eval_node(*be.right) {
            Some(v) => v,
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

        Some(val)
    }
}
