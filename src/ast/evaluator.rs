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

    pub fn eval(&mut self, node: ASTNode) -> f64 {
        match node {
            ASTNode::Number(value) => return value,
            ASTNode::BinaryExpression(be) => self.eval_binary_expression(be),
            ASTNode::UnaryExpression(n) => self.eval_unary_expression(*n),
            ASTNode::VariableExpression(ve) => {
                self.eval_variable_statement(ve);
                return 0.0;
            }
            ASTNode::Variable(name) => match self.symbol_table.get(&name) {
                Some(val) => val.to_owned(),
                None => panic!("unknown variable {}", name),
            },
        }
    }

    fn eval_variable_statement(&mut self, node: VariableExpression) {
        let val = self.eval(*node.value);
        self.symbol_table.insert(node.name, val);
    }

    fn eval_unary_expression(&mut self, node: ASTNode) -> f64 {
        -self.eval(node)
    }

    fn eval_binary_expression(&mut self, be: BinaryExpression) -> f64 {
        let l = self.eval(*be.left);
        let r = self.eval(*be.right);

        match be.operator {
            TokenType::Plus => l + r,
            TokenType::Minus => l - r,
            TokenType::Asterisk => l * r,
            TokenType::Slash => l / r,
            TokenType::Carat => l.powf(r),
            _ => panic!(""),
        }
    }
}
