use super::ast::{
    ASTNode, BinaryExpression, BlockStatement, FunctionCall, FunctionExpression, IfStatement,
    VariableExpression,
};
use super::scope::ScopeKind;
use super::symbol::Symbol;
use super::symbol_table::SymbolTable;
use super::util;
use crate::common::bash;
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
            ASTNode::BinaryExpression(be) => self.eval_binary_expression(be),
            ASTNode::UnaryExpression(n) => self.eval_unary_expression(*n),
            ASTNode::VariableExpression(ve) => {
                self.eval_variable_expression(ve);
                None
            }
            ASTNode::Variable(name) => Some(self.get_symbol(&name)),
            ASTNode::FunctionExpression(fe) => {
                self.symbol_table
                    .insert(&fe.name, Symbol::Function(fe.clone()));
                None
            }
            ASTNode::FunctionCall(fc) => self.eval_function_call(fc),
            ASTNode::IfStatement(is) => {
                self.eval_if_statement(is);
                None
            }
            ASTNode::BlockStatement(bs) => self.eval_block_statement(bs),
            ASTNode::ReturnExpression(expr) => self.eval_node(*expr),
            ASTNode::Number(value) => Some(Symbol::Number(value)),
            ASTNode::Boolean(value) => Some(Symbol::Boolean(value)),
            ASTNode::String(value) => Some(Symbol::String(value)),
            ASTNode::Command(cmd) => Some(self.eval_command(*cmd)),
            _ => None,
        }
    }

    fn get_symbol(&self, name: &str) -> Symbol {
        match self.symbol_table.get(&name) {
            Some(symbol) => symbol.clone(),
            None => {
                panic!("undeclared variable '{}'", name);
            }
        }
    }

    fn eval_command(&mut self, tokens: Vec<ASTNode>) -> Symbol {
        let mut cmd_string = "".to_owned();
        for node in tokens {
            if let Some(sym) = self.eval_node(node) {
                cmd_string.push_str(sym.to_string().as_str());
            }
        }

        let output = bash::run_cmd(&cmd_string);
        Symbol::String(output)
    }

    fn eval_block_statement(&mut self, block_statement: BlockStatement) -> Option<Symbol> {
        for node in *block_statement.body {
            match node {
                ASTNode::ReturnExpression(expr) => return self.eval_node(*expr),
                _ => self.eval_node(node),
            };
        }

        None
    }

    fn eval_if_statement(&mut self, if_statement: IfStatement) {
        let passed = match self.eval_node(*if_statement.condition) {
            Some(sym) => match sym {
                Symbol::Number(num) => num != 0.0,
                Symbol::Boolean(b) => b,
                _ => false,
            },
            None => false,
        };

        if passed {
            self.symbol_table.push_scope(ScopeKind::ConditionalBlock);
            self.eval_node(*if_statement.consequence);
            self.symbol_table.pop_scope();
        } else if let Some(alternative) = if_statement.alternative {
            self.symbol_table.push_scope(ScopeKind::ConditionalBlock);
            self.eval_node(*alternative);
            self.symbol_table.pop_scope();
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
        for (arg_name, arg_value) in func_expr.args.iter().zip(func_call.args.iter()) {
            let symbol = match self.eval_node(arg_value.clone()) {
                Some(symbol) => symbol,
                None => panic!("invalid function argument for {}", func_expr.name),
            };
            args.push((arg_name, symbol));
        }

        self.symbol_table.push_scope(ScopeKind::FunctionBlock);

        for (arg_name, arg_value) in args {
            self.symbol_table.insert(arg_name, arg_value);
        }
    }

    fn eval_function_call(&mut self, func_call: FunctionCall) -> Option<Symbol> {
        let func_expr = match self.get_symbol(&func_call.name) {
            Symbol::Function(f) => f.clone(),
            _ => return None,
        };

        self.validate_function_call(&func_call, &func_expr);

        self.push_function(&func_call, &func_expr);
        let res = self.eval_node(*func_expr.body);
        self.symbol_table.pop_scope();

        res
    }

    fn eval_variable_expression(&mut self, node: VariableExpression) {
        if let Some(symbol) = self.eval_node(*node.rhs) {
            self.symbol_table.insert(&node.name, symbol);
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
        let left_symbol = match self.eval_node(*be.left) {
            Some(s) => s,
            None => return None,
        };

        let right_symbol = match self.eval_node(*be.right) {
            Some(s) => s,
            None => return None,
        };

        let result_symbol = match &be.operator {
            op if util::is_comparative_operator(op) => {
                self.compare(&left_symbol, op, &right_symbol)
            }
            _ => self.eval_math_expression(&left_symbol, &be.operator, &right_symbol),
        };

        Some(result_symbol)
    }

    fn eval_math_expression(&self, left: &Symbol, operator: &TokenType, right: &Symbol) -> Symbol {
        let (l, r) = match (left, right) {
            (Symbol::Number(ln), Symbol::Number(rn)) => (ln, rn),
            _ => panic!(
                "{:?} {:?} {:?}: can only perform mathematical expressions on numbers",
                left, operator, right
            ),
        };

        let res = match operator {
            TokenType::Plus => l + r,
            TokenType::Minus => l - r,
            TokenType::Asterisk => l * r,
            TokenType::ForwardSlash => l / r,
            TokenType::Carat => l.powf(*r),
            _ => panic!("invalid operator {:?}", operator),
        };

        Symbol::Number(res)
    }

    fn compare(&self, left: &Symbol, operator: &TokenType, right: &Symbol) -> Symbol {
        match (left, right) {
            (Symbol::Number(ln), Symbol::Number(rn)) => self.compare_number(*ln, operator, *rn),
            (Symbol::Boolean(lb), Symbol::Boolean(rb)) => self.compare_bool(*lb, operator, *rb),
            _ => panic!("{:?} {:?} {:?}: type mismatch", left, operator, right),
        }
    }

    fn compare_bool(&self, left: bool, operator: &TokenType, right: bool) -> Symbol {
        let bool_result = match operator {
            TokenType::DoubleEquals => left == right,
            TokenType::NotEquals => left != right,
            TokenType::And => left && right,
            TokenType::Or => left || right,
            _ => panic!(
                "{:?} {:?} {:?}: unable to compare booleans",
                left, operator, right
            ),
        };

        Symbol::Boolean(bool_result)
    }

    fn compare_number(&self, left: f64, operator: &TokenType, right: f64) -> Symbol {
        let res = match operator {
            TokenType::DoubleEquals => left == right,
            TokenType::NotEquals => left != right,
            TokenType::GreaterThan => left > right,
            TokenType::LessThan => left < right,
            TokenType::GreaterThanOrEqualTo => left >= right,
            TokenType::LessThanOrEqualTo => left <= right,
            _ => panic!("expected a comparison"),
        };

        Symbol::Boolean(res)
    }
}
