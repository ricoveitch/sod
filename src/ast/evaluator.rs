use super::ast::{ASTNode, BinaryExpression, FunctionCall, FunctionExpression, VariableExpression};
use super::symbol::Symbol;
use super::symbol_table::SymbolTable;
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
                    .insert(&fe.name, Symbol::Function(fe.clone()));
                None
            }
            ASTNode::FunctionCall(fc) => self.eval_function_call(fc),
            _ => None,
        }
    }

    fn validate_function_call(&self, func_call: &FunctionCall, func_expr: &FunctionExpression) {
        if func_call.args.len() < func_expr.args.len() {
            panic!(
                "{} missing function args expected {} received {}",
                func_expr.name,
                func_expr.args.len(),
                func_call.args.len()
            )
        }
    }

    fn push_function(&mut self, func_call: &FunctionCall, func_expr: &FunctionExpression) {
        let mut args = vec![];
        // evaluate any variables in args
        for (arg_name, arg_value) in func_expr.args.iter().zip(func_call.args.iter()) {
            let value = match arg_value {
                Symbol::Variable(var_name) => self.symbol_table.get(var_name).clone(),
                _ => arg_value.clone(),
            };
            args.push((arg_name, value));
        }

        self.symbol_table.push_scope(&func_expr.name);

        for (arg_name, arg_value) in args {
            self.symbol_table.insert(arg_name, arg_value);
        }
    }

    fn eval_function_call(&mut self, func_call: FunctionCall) -> Option<Symbol> {
        let func_expr = match self.symbol_table.get(&func_call.name) {
            Symbol::Function(f) => f.clone(),
            _ => return None,
        };

        self.validate_function_call(&func_call, &func_expr);
        self.push_function(&func_call, &func_expr);

        for line in *func_expr.body {
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
