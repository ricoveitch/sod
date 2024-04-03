use super::ast::{
    ASTNode, BinaryExpression, BlockStatement, FunctionCall, FunctionExpression, IfStatement,
    MemberExpression, MemberExpressionKind, VariableExpression,
};
use super::scope::ScopeKind;
use super::symbol::{self, List, Symbol};
use super::symbol_table::SymbolTable;
use crate::common::bash;
use crate::lexer::token::TokenType;
use crate::new_string_symbol;

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
            ASTNode::MemberExpression(me) => self.eval_member_expression(me),
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
            ASTNode::String(value) => Some(new_string_symbol!(value)),
            ASTNode::List(nodes) => Some(self.eval_list(*nodes)),
            ASTNode::Command(cmd) => Some(self.eval_command(*cmd)),
            ASTNode::Identifier(ident) => Some(self.get_symbol(&ident).clone()),
            ASTNode::Program(_) => None,
        }
    }

    fn get_symbol(&self, name: &str) -> &Symbol {
        match self.symbol_table.get(&name) {
            Some(symbol) => symbol,
            None => {
                panic!("undeclared variable '{}'", name);
            }
        }
    }

    fn get_symbol_mut(&mut self, name: &str) -> &mut Symbol {
        match self.symbol_table.get_mut(&name) {
            Some(symbol) => symbol,
            None => {
                panic!("undeclared variable '{}'", name);
            }
        }
    }

    fn eval_member_expression(&mut self, me: MemberExpression) -> Option<Symbol> {
        match *me.kind {
            MemberExpressionKind::Index(expr) => {
                Some(self.eval_member_index(me.identifier.as_str(), expr))
            }
            MemberExpressionKind::Call(call) => self.eval_symbol_call(me.identifier.as_str(), call),
        }
    }

    fn visit_function_args(&mut self, args: Vec<ASTNode>) -> Vec<Symbol> {
        let mut result = vec![];
        for node in args {
            match self.eval_node(node) {
                Some(symbol) => result.push(symbol),
                None => panic!("TODO: handle None type"),
            };
        }

        result
    }

    fn eval_symbol_call(&mut self, indent: &str, call: FunctionCall) -> Option<Symbol> {
        let args = self.visit_function_args(call.args);
        let call = call.name.as_str();
        let symbol = self.get_symbol_mut(&indent);

        match symbol {
            Symbol::List(list) => list.call(call, args),
            Symbol::String(ss) => ss.call(call, args),
            _ => panic!("{} has no member {}", symbol, call),
        }
    }

    fn visit_index_expression(&mut self, expression: ASTNode) -> usize {
        let expr_symbol = match self.eval_node(expression) {
            Some(s) => s,
            None => panic!("indices must be numbers"),
        };

        match expr_symbol {
            Symbol::Number(index) => index as usize,
            _ => panic!("indices must be numbers"),
        }
    }

    fn eval_member_index(&mut self, member: &str, expression: ASTNode) -> Symbol {
        let index = self.visit_index_expression(expression);

        match self.get_symbol(&member) {
            Symbol::List(list) => list.get(index).clone(),
            Symbol::String(string) => Symbol::String(string.get(index)),
            _ => panic!("object {} is not indexable", member),
        }
    }

    fn eval_member_index_mut(&mut self, member: &str, expression: ASTNode) -> &mut Symbol {
        let index = self.visit_index_expression(expression);

        match self.get_symbol_mut(&member) {
            Symbol::List(list) => list.get_mut(index),
            _ => panic!("object {} is not indexable", member),
        }
    }

    fn eval_list(&mut self, nodes: Vec<ASTNode>) -> Symbol {
        let mut items = vec![];
        for node in nodes {
            match self.eval_node(node) {
                Some(symbol) => items.push(symbol),
                None => panic!("invalid expression in list"),
            }
        }

        return Symbol::List(List { items });
    }

    fn eval_command(&mut self, tokens: Vec<ASTNode>) -> Symbol {
        let mut cmd_string = "".to_owned();
        for node in tokens {
            if let Some(sym) = self.eval_node(node) {
                cmd_string.push_str(sym.to_string().as_str());
            }
        }

        let output = bash::run_cmd(&cmd_string);
        new_string_symbol!(output)
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
            Some(sym) => sym.is_truthy(),
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

    fn push_function(&mut self, func_call: FunctionCall, func_expr: &FunctionExpression) {
        let arg_values = self.visit_function_args(func_call.args);
        let mut args = vec![];
        for (name, value) in func_expr.args.iter().zip(arg_values.iter()) {
            args.push((name, value.clone()));
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

        self.push_function(func_call, &func_expr);
        let res = self.eval_node(*func_expr.body);
        self.symbol_table.pop_scope();

        res
    }

    fn eval_variable_expression(&mut self, node: VariableExpression) {
        let rhs = match self.eval_node(*node.rhs) {
            Some(s) => s,
            None => panic!("TODO: complete when adding None type."),
        };

        match *node.lhs {
            ASTNode::Identifier(ident) => self.symbol_table.insert(&ident, rhs),
            ASTNode::MemberExpression(me) => match *me.kind {
                MemberExpressionKind::Index(expr) => {
                    let lhs_symbol = self.eval_member_index_mut(me.identifier.as_str(), expr);
                    *lhs_symbol = rhs;
                }
                _ => unimplemented!("member expression must use an index"),
            },
            _ => unimplemented!("left hand side must be identifier or member expression"),
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

        if be.operator == TokenType::And && !left_symbol.is_truthy() {
            return None;
        }

        if be.operator == TokenType::Or && left_symbol.is_truthy() {
            return Some(left_symbol);
        }

        let right_symbol = match self.eval_node(*be.right) {
            Some(s) => s,
            None => return None,
        };

        Some(symbol::eval_binary_expression(
            &left_symbol,
            &be.operator,
            &right_symbol,
        ))
    }
}
