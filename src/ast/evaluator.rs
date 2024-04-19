use super::ast::{
    self, ASTNode, BinaryExpression, BlockStatement, CallExpression, ForStatement,
    FunctionStatement, IfStatement, IndexExpression, MemberExpression, RangeExpression,
    TemplateString, VariableExpression,
};
use crate::common::bash;
use crate::lexer::token::TokenType;
use crate::new_string_symbol;
use crate::symbol::scope::ScopeKind;
use crate::symbol::symbol::{self, List, Range, Symbol};
use crate::symbol::table::SymbolTable;

enum SymbolRef<'a> {
    MutRef(&'a mut Symbol),
    Value(Symbol),
}

pub struct ASTEvaluator {
    symbol_table: SymbolTable,
}

impl ASTEvaluator {
    pub fn new(argv: Vec<String>) -> Self {
        let global_vars = symbol::get_global_vars(argv);
        Self {
            symbol_table: SymbolTable::from(global_vars),
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
            ASTNode::MemberExpression(me) => Some(self.visit_member_expression(me).clone()),
            ASTNode::IndexExpression(ie) => Some(self.visit_index_expression(ie)),
            ASTNode::FunctionStatement(fs) => {
                self.symbol_table
                    .set(&fs.name.clone(), Symbol::Function(Box::new(fs)));
                None
            }
            ASTNode::CallExpression(fc) => self.eval_call_expression(fc),
            ASTNode::IfStatement(is) => {
                self.eval_if_statement(is);
                None
            }

            ASTNode::BlockStatement(bs) => self.eval_block_statement(bs),
            ASTNode::ReturnStatement(expr) => self.eval_node(*expr),
            ASTNode::ForStatement(fs) => {
                self.eval_for_statement(fs);
                None
            }

            ASTNode::Number(value) => Some(Symbol::Number(value)),
            ASTNode::Boolean(value) => Some(Symbol::Boolean(value)),
            ASTNode::String(value) => Some(new_string_symbol!(value)),
            ASTNode::TemplateString(ts) => Some(self.visit_template_string(ts)),
            ASTNode::List(nodes) => Some(self.eval_list(*nodes)),
            ASTNode::None => Some(Symbol::None),
            ASTNode::RangeExpression(range_expr) => {
                Some(Symbol::Range(self.visit_range_expression(range_expr)))
            }

            ASTNode::Command(cmd) => Some(self.eval_command(*cmd)),
            // TODO: allow returning reference to a symbol in the future.
            ASTNode::Identifier(ident) => Some(self.get_symbol(&ident).clone()),
            ASTNode::Program(_) => None,
        }
    }

    fn visit_node_mut(&mut self, node: ASTNode) -> SymbolRef {
        match node {
            ASTNode::MemberExpression(me) => {
                SymbolRef::MutRef(self.visit_member_expression_mut(me))
            }
            ASTNode::Identifier(ident) => SymbolRef::MutRef(self.get_symbol_mut(&ident)),
            ASTNode::CallExpression(ce) => match self.eval_call_expression(ce) {
                Some(value) => SymbolRef::Value(value),
                None => panic!("call has no value"),
            },
            _ => panic!("not mutable"),
        }
    }

    fn get_symbol(&self, name: &str) -> &Symbol {
        match self.symbol_table.get(&name) {
            Some(symbol) => symbol,
            None => {
                panic!("'{}' is not defined", name);
            }
        }
    }

    fn get_symbol_mut(&mut self, name: &str) -> &mut Symbol {
        match self.symbol_table.get_mut(&name) {
            Some(symbol) => symbol,
            None => {
                panic!("'{}' is not defined", name);
            }
        }
    }

    fn visit_template_string(&self, template_string: TemplateString) -> Symbol {
        let mut res = "".to_string();
        for token in template_string.tokens {
            let sub_str = match token {
                ast::TemplateToken::Expression(expr) => {
                    let symbol = self.get_symbol(expr.as_str());
                    symbol.to_string()
                }
                ast::TemplateToken::Literal(s) => s,
            };
            res.push_str(sub_str.as_str());
        }

        new_string_symbol!(res)
    }

    fn visit_range_expression(&mut self, range_expr: RangeExpression) -> Range {
        let mut visit_range_prop = |node: ASTNode, label: &str| match self.eval_node(node) {
            Some(symbol) => match symbol {
                Symbol::Number(num) => num as i32,
                _ => panic!("range {} must be a number", label),
            },
            None => panic!("invalid range"),
        };

        let start = visit_range_prop(*range_expr.start, "start");
        let end = visit_range_prop(*range_expr.end, "end");
        let increment = if let Some(inc) = range_expr.increment {
            Some(visit_range_prop(*inc, "increment"))
        } else {
            None
        };

        Range::new(start, end, increment)
    }

    fn visit_iterable(&mut self, iterable: ast::Iterable) -> Box<dyn Iterator<Item = Symbol>> {
        match iterable {
            ast::Iterable::RangeExpression(re) => Box::new(self.visit_range_expression(re)),
            ast::Iterable::Collection(node) => match self.eval_node(node) {
                Some(symbol) => match symbol {
                    Symbol::List(list) => Box::new(list.items.into_iter()),
                    Symbol::String(ss) => Box::new(ss.into_iter()),
                    _ => panic!("{} is not iterable", symbol.kind()),
                },
                None => panic!("iterator not found"),
            },
        }
    }

    fn eval_for_statement(&mut self, for_statement: ForStatement) {
        let iterable = self.visit_iterable(*for_statement.iterable);
        self.symbol_table.push_scope(ScopeKind::ForBlock);

        for symbol in iterable {
            self.symbol_table
                .set(for_statement.variable.as_str(), symbol);
            self.eval_node(*for_statement.body.clone());
        }

        self.symbol_table.pop_scope();
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
        print!("{}", output);
        new_string_symbol!(output)
    }

    fn eval_block_statement(&mut self, block_statement: BlockStatement) -> Option<Symbol> {
        for node in *block_statement.body {
            match node {
                ASTNode::ReturnStatement(expr) => return self.eval_node(*expr),
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

    fn validate_function_call(&self, func_call: &CallExpression, func_expr: &FunctionStatement) {
        if func_call.args.len() < func_expr.args.len() {
            panic!(
                "{} missing function args expected {} received {}",
                func_expr.name,
                func_expr.args.len(),
                func_call.args.len()
            )
        }
    }

    fn push_function(&mut self, func_call: CallExpression, func_expr: &FunctionStatement) {
        let arg_values = self.visit_function_args(func_call.args);
        let mut args = vec![];
        for (name, value) in func_expr.args.iter().zip(arg_values.iter()) {
            args.push((name, value.clone()));
        }

        self.symbol_table.push_scope(ScopeKind::FunctionBlock);

        for (arg_name, arg_value) in args {
            self.symbol_table.set(arg_name, arg_value);
        }
    }

    fn visit_function(&mut self, func_name: &str, call_expr: CallExpression) -> Option<Symbol> {
        let func_expr = match self.get_symbol(func_name) {
            Symbol::Function(f) => f.clone(),
            _ => return None,
        };

        self.validate_function_call(&call_expr, &func_expr);

        self.push_function(call_expr, &func_expr);
        let res = self.eval_node(*func_expr.body);
        self.symbol_table.pop_scope();

        res
    }

    fn visit_member_expression_call(
        &mut self,
        member_expr: MemberExpression,
        ast_args: Vec<ASTNode>,
    ) -> Option<Symbol> {
        let args = self.visit_function_args(ast_args);
        let call = member_expr.property.as_str();

        match self.visit_node_mut(*member_expr.base) {
            SymbolRef::MutRef(symbol) => symbol.call(call, args),
            SymbolRef::Value(mut symbol) => symbol.call(call, args),
        }
    }

    fn eval_call_expression(&mut self, call_expr: CallExpression) -> Option<Symbol> {
        match *call_expr.base {
            ASTNode::Identifier(ref fname) => {
                self.visit_function(fname.clone().as_str(), call_expr)
            }
            ASTNode::MemberExpression(me) => self.visit_member_expression_call(me, call_expr.args),
            _ => unimplemented!("object is not callable"),
        }
    }

    fn eval_index(&mut self, expression: ASTNode) -> usize {
        let expr_symbol = match self.eval_node(expression) {
            Some(s) => s,
            None => panic!("indices must be numbers"),
        };

        // TODO: later use u64 instead for [-1] list access?
        match expr_symbol {
            Symbol::Number(index) => index as usize,
            _ => panic!("indices must be numbers"),
        }
    }

    fn visit_index_expression(&mut self, index_expr: IndexExpression) -> Symbol {
        let index = self.eval_index(*index_expr.index);
        let symbol = self.eval_node(*index_expr.base).unwrap();

        match symbol {
            Symbol::List(list) => list.get(index).clone(),
            Symbol::String(ss) => ss.get(index),
            _ => panic!("{} is not indexable", symbol.kind()),
        }
    }

    fn visit_index_expression_mut(&mut self, index_expr: IndexExpression) -> &mut Symbol {
        let index = self.eval_index(*index_expr.index);
        match self.visit_node_mut(*index_expr.base) {
            SymbolRef::MutRef(mr) => mr.get_index_mut(index),
            //SymbolRef::Value(mut val) => val.get_index_mut(index),
            _ => unimplemented!("by value index mutation"),
        }
    }

    fn visit_member_expression(&mut self, member_expr: MemberExpression) -> Symbol {
        let symbol = match *member_expr.base {
            ASTNode::Identifier(ident) => self.get_symbol(ident.as_str()),
            _ => unimplemented!("TODO"),
        };

        match symbol {
            Symbol::Object(obj) => obj.get(member_expr.property.as_str()).clone(),
            _ => panic!("{} has no property {}", symbol.kind(), member_expr.property),
        }
    }

    fn visit_member_expression_mut(&mut self, member_expr: MemberExpression) -> &mut Symbol {
        let symbol = match *member_expr.base {
            ASTNode::Identifier(ident) => self.get_symbol_mut(ident.as_str()),
            _ => unimplemented!("object not supported"),
        };

        match symbol {
            Symbol::Object(obj) => obj.get_mut(member_expr.property.as_str()),
            _ => panic!("{} has no property {}", symbol.kind(), member_expr.property),
        }
    }

    fn eval_variable_expression(&mut self, node: VariableExpression) {
        let rhs = match self.eval_node(*node.rhs) {
            Some(s) => s,
            None => panic!("right hand side not found"),
        };

        match *node.lhs {
            ASTNode::Identifier(ident) => self.symbol_table.set(&ident, rhs),
            ASTNode::IndexExpression(ie) => {
                let lhs_symbol = self.visit_index_expression_mut(ie);
                *lhs_symbol = rhs;
            }
            _ => unimplemented!("object assignment"),
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
            return Some(left_symbol);
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
