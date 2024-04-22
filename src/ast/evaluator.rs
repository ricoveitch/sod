use super::ast::{
    self, ASTNode, BinaryExpression, BlockStatement, CallExpression, ForStatement,
    FunctionStatement, IfStatement, IndexExpression, MemberExpression, RangeExpression,
    TemplateString, VariableExpression,
};
use crate::commands;
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

    pub fn eval(&mut self, program: ASTNode) -> Result<Vec<Option<Symbol>>, String> {
        let mut prog_results = vec![];
        match program {
            ASTNode::Program(root) => {
                for line in *root {
                    prog_results.push(self.eval_node(line)?);
                }
                Ok(prog_results)
            }
            _ => Err("expected program".to_string()),
        }
    }

    fn eval_node(&mut self, node: ASTNode) -> Result<Option<Symbol>, String> {
        let option = match node {
            ASTNode::BinaryExpression(be) => self.eval_binary_expression(be)?,
            ASTNode::UnaryExpression(n) => self.eval_unary_expression(*n)?,
            ASTNode::VariableExpression(ve) => {
                self.eval_variable_expression(ve)?;
                None
            }
            ASTNode::MemberExpression(me) => Some(self.visit_member_expression(me)?.clone()),
            ASTNode::IndexExpression(ie) => Some(self.visit_index_expression(ie)?),
            ASTNode::FunctionStatement(fs) => {
                self.symbol_table
                    .set(&fs.name.clone(), Symbol::Function(Box::new(fs)));
                None
            }
            ASTNode::CallExpression(fc) => Some(self.eval_call_expression(fc)?),
            ASTNode::IfStatement(is) => {
                self.eval_if_statement(is)?;
                None
            }

            ASTNode::BlockStatement(bs) => Some(self.eval_block_statement(bs)?),
            ASTNode::ReturnStatement(expr) => self.eval_node(*expr)?,
            ASTNode::ForStatement(fs) => {
                self.eval_for_statement(fs)?;
                None
            }

            ASTNode::Number(value) => Some(Symbol::Number(value)),
            ASTNode::Boolean(value) => Some(Symbol::Boolean(value)),
            ASTNode::String(value) => Some(new_string_symbol!(value)),
            ASTNode::TemplateString(ts) => Some(self.visit_template_string(ts)?),
            ASTNode::List(nodes) => Some(self.eval_list(*nodes)?),
            ASTNode::None => Some(Symbol::None),
            ASTNode::RangeExpression(range_expr) => {
                Some(Symbol::Range(self.visit_range_expression(range_expr)?))
            }

            ASTNode::Command(cmd) => Some(self.eval_command(*cmd)?),
            // TODO: allow returning reference to a symbol in the future.
            ASTNode::Identifier(ident) => Some(self.get_symbol(&ident)?.clone()),
            ASTNode::Program(_) => None,
        };

        Ok(option)
    }

    fn visit_node_mut(&mut self, node: ASTNode) -> Result<SymbolRef, String> {
        let res = match node {
            ASTNode::MemberExpression(me) => {
                SymbolRef::MutRef(self.visit_member_expression_mut(me)?)
            }
            ASTNode::Identifier(ident) => SymbolRef::MutRef(self.get_symbol_mut(&ident)?),
            ASTNode::CallExpression(ce) => SymbolRef::Value(self.eval_call_expression(ce)?),
            _ => return Err(format!("not mutable")),
        };

        Ok(res)
    }

    fn get_symbol(&self, name: &str) -> Result<&Symbol, String> {
        match self.symbol_table.get(&name) {
            Some(symbol) => Ok(symbol),
            None => Err(format!("'{}' is not defined", name)),
        }
    }

    fn get_symbol_mut(&mut self, name: &str) -> Result<&mut Symbol, String> {
        match self.symbol_table.get_mut(&name) {
            Some(symbol) => Ok(symbol),
            None => Err(format!("'{}' is not defined", name)),
        }
    }

    fn visit_template_string(&self, template_string: TemplateString) -> Result<Symbol, String> {
        let mut res = "".to_string();
        for token in template_string.tokens {
            let sub_str = match token {
                ast::TemplateToken::Expression(expr) => {
                    let symbol = self.get_symbol(expr.as_str())?;
                    symbol.to_string()
                }
                ast::TemplateToken::Literal(s) => s,
            };
            res.push_str(sub_str.as_str());
        }

        Ok(new_string_symbol!(res))
    }

    fn visit_range_expression(&mut self, range_expr: RangeExpression) -> Result<Range, String> {
        let mut visit_range_prop = |node: ASTNode, label: &str| -> Result<i32, String> {
            match self.eval_node(node)? {
                Some(symbol) => match symbol {
                    Symbol::Number(num) => Ok(num as i32),
                    _ => Err(format!(
                        "range {} must be a number, found {}",
                        label,
                        symbol.kind()
                    )),
                },
                None => Err(format!("invalid range")),
            }
        };

        let start = visit_range_prop(*range_expr.start, "start")?;
        let end = visit_range_prop(*range_expr.end, "end")?;
        let increment = if let Some(inc) = range_expr.increment {
            Some(visit_range_prop(*inc, "increment")?)
        } else {
            None
        };

        Ok(Range::new(start, end, increment))
    }

    fn visit_iterable(
        &mut self,
        iterable: ast::Iterable,
    ) -> Result<Box<dyn Iterator<Item = Symbol>>, String> {
        match iterable {
            ast::Iterable::RangeExpression(re) => {
                let iterator = self.visit_range_expression(re)?;
                Ok(Box::new(iterator))
            }
            ast::Iterable::Collection(node) => match self.eval_node(node)? {
                Some(symbol) => match symbol {
                    Symbol::List(list) => Ok(Box::new(list.items.into_iter())),
                    Symbol::String(ss) => Ok(Box::new(ss.into_iter())),
                    Symbol::Range(r) => Ok(Box::new(r.into_iter())),
                    _ => Err(format!("{} is not iterable", symbol.kind())),
                },
                None => Err("iterator not found".to_string()),
            },
        }
    }

    fn eval_for_statement(&mut self, for_statement: ForStatement) -> Result<(), String> {
        let iterable = self.visit_iterable(*for_statement.iterable)?;
        self.symbol_table.push_scope(ScopeKind::ForBlock);

        for symbol in iterable {
            self.symbol_table
                .set(for_statement.variable.as_str(), symbol);
            self.eval_node(*for_statement.body.clone())?;
        }

        self.symbol_table.pop_scope();
        Ok(())
    }

    fn visit_function_args(&mut self, args: Vec<ASTNode>) -> Result<Vec<Symbol>, String> {
        let mut result = vec![];
        for node in args {
            match self.eval_node(node)? {
                Some(symbol) => result.push(symbol),
                None => return Err(format!("TODO: handle None type")),
            };
        }

        Ok(result)
    }

    fn eval_list(&mut self, nodes: Vec<ASTNode>) -> Result<Symbol, String> {
        let mut items = vec![];
        for node in nodes {
            match self.eval_node(node)? {
                Some(symbol) => items.push(symbol),
                None => return Err(format!("invalid expression in list")),
            }
        }

        return Ok(Symbol::List(List { items }));
    }

    fn eval_command(&mut self, tokens: Vec<ASTNode>) -> Result<Symbol, String> {
        let mut cmd_string = "".to_owned();
        for node in tokens {
            if let Some(sym) = self.eval_node(node)? {
                cmd_string.push_str(sym.to_string().as_str());
            }
        }

        let output = commands::run_cmd(&cmd_string);
        print!("{}", output);
        Ok(new_string_symbol!(output))
    }

    fn eval_block_statement(&mut self, block_statement: BlockStatement) -> Result<Symbol, String> {
        for node in *block_statement.body {
            match node {
                ASTNode::ReturnStatement(expr) => {
                    return match self.eval_node(*expr)? {
                        Some(s) => Ok(s),
                        None => Ok(Symbol::None),
                    }
                }
                _ => self.eval_node(node)?,
            };
        }

        Ok(Symbol::None)
    }

    fn eval_if_statement(&mut self, if_statement: IfStatement) -> Result<(), String> {
        let passed = match self.eval_node(*if_statement.condition)? {
            Some(sym) => sym.is_truthy(),
            None => false,
        };

        if passed {
            self.symbol_table.push_scope(ScopeKind::ConditionalBlock);
            self.eval_node(*if_statement.consequence)?;
            self.symbol_table.pop_scope();
        } else if let Some(alternative) = if_statement.alternative {
            self.symbol_table.push_scope(ScopeKind::ConditionalBlock);
            self.eval_node(*alternative)?;
            self.symbol_table.pop_scope();
        }

        Ok(())
    }

    fn validate_function_call(
        &self,
        func_call: &CallExpression,
        func_expr: &FunctionStatement,
    ) -> Result<(), String> {
        if func_call.args.len() < func_expr.args.len() {
            return Err(format!(
                "{} missing function args expected {} received {}",
                func_expr.name,
                func_expr.args.len(),
                func_call.args.len()
            ));
        }

        Ok(())
    }

    fn push_function(
        &mut self,
        func_call: CallExpression,
        func_expr: &FunctionStatement,
    ) -> Result<(), String> {
        let arg_values = self.visit_function_args(func_call.args)?;
        let mut args = vec![];
        for (name, value) in func_expr.args.iter().zip(arg_values.iter()) {
            args.push((name, value.clone()));
        }

        self.symbol_table.push_scope(ScopeKind::FunctionBlock);

        for (arg_name, arg_value) in args {
            self.symbol_table.set(arg_name, arg_value);
        }

        Ok(())
    }

    fn visit_function(
        &mut self,
        func_name: &str,
        call_expr: CallExpression,
    ) -> Result<Symbol, String> {
        let func_statement = match self.get_symbol(func_name)? {
            Symbol::Function(f) => f.clone(),
            _ => return Ok(Symbol::None),
        };

        self.validate_function_call(&call_expr, &func_statement)?;

        self.push_function(call_expr, &func_statement)?;
        let res = self.eval_node(*func_statement.body)?;
        self.symbol_table.pop_scope();

        match res {
            Some(symbol) => Ok(symbol),
            None => Ok(Symbol::None),
        }
    }

    fn visit_member_expression_call(
        &mut self,
        member_expr: MemberExpression,
        ast_args: Vec<ASTNode>,
    ) -> Result<Symbol, String> {
        let args = self.visit_function_args(ast_args)?;
        let call = member_expr.property.as_str();

        let symbol = match self.visit_node_mut(*member_expr.base)? {
            SymbolRef::MutRef(symbol) => symbol.call(call, args)?,
            SymbolRef::Value(mut symbol) => symbol.call(call, args)?,
        };

        Ok(symbol)
    }

    fn eval_call_expression(&mut self, call_expr: CallExpression) -> Result<Symbol, String> {
        match *call_expr.base {
            ASTNode::Identifier(ref fname) => {
                self.visit_function(fname.clone().as_str(), call_expr)
            }
            ASTNode::MemberExpression(me) => self.visit_member_expression_call(me, call_expr.args),
            _ => unimplemented!("object is not callable"),
        }
    }

    fn eval_index(&mut self, expression: ASTNode) -> Result<usize, String> {
        let expr_symbol = match self.eval_node(expression)? {
            Some(s) => s,
            None => return Err("indices must be numbers".to_string()),
        };

        // TODO: later use u64 instead for [-1] list access?
        match expr_symbol {
            Symbol::Number(index) => Ok(index as usize),
            _ => Err("indices must be numbers".to_string()),
        }
    }

    fn visit_index_expression(&mut self, index_expr: IndexExpression) -> Result<Symbol, String> {
        let index = self.eval_index(*index_expr.index)?;
        let symbol = self.eval_node(*index_expr.base)?.unwrap();

        match symbol {
            Symbol::List(list) => Ok(list.get(index)?.clone()),
            Symbol::String(ss) => Ok(ss.get(index)?),
            _ => Err(format!("{} is not indexable", symbol.kind())),
        }
    }

    fn visit_index_expression_mut(
        &mut self,
        index_expr: IndexExpression,
    ) -> Result<&mut Symbol, String> {
        let index = self.eval_index(*index_expr.index)?;
        match self.visit_node_mut(*index_expr.base)? {
            SymbolRef::MutRef(mr) => Ok(mr.get_index_mut(index)?),
            //SymbolRef::Value(mut val) => val.get_index_mut(index),
            _ => unimplemented!("by value index mutation"),
        }
    }

    fn visit_member_expression(&mut self, member_expr: MemberExpression) -> Result<Symbol, String> {
        let symbol = match *member_expr.base {
            ASTNode::Identifier(ident) => self.get_symbol(ident.as_str())?,
            _ => unimplemented!("TODO"),
        };

        match symbol {
            Symbol::Object(obj) => Ok(obj.get(member_expr.property.as_str()).clone()),
            _ => Err(format!(
                "{} has no property {}",
                symbol.kind(),
                member_expr.property
            )),
        }
    }

    fn visit_member_expression_mut(
        &mut self,
        member_expr: MemberExpression,
    ) -> Result<&mut Symbol, String> {
        let symbol = match *member_expr.base {
            ASTNode::Identifier(ident) => self.get_symbol_mut(ident.as_str())?,
            _ => unimplemented!("object not supported"),
        };

        match symbol {
            Symbol::Object(obj) => Ok(obj.get_mut(member_expr.property.as_str())),
            _ => Err(format!(
                "{} has no property {}",
                symbol.kind(),
                member_expr.property
            )),
        }
    }

    fn eval_variable_expression(&mut self, node: VariableExpression) -> Result<(), String> {
        let rhs = match self.eval_node(*node.rhs)? {
            Some(s) => s,
            None => return Err(format!("right hand side not found")),
        };

        match *node.lhs {
            ASTNode::Identifier(ident) => self.symbol_table.set(&ident, rhs),
            ASTNode::IndexExpression(ie) => {
                let lhs_symbol = self.visit_index_expression_mut(ie)?;
                *lhs_symbol = rhs;
            }
            _ => unimplemented!("object assignment"),
        };

        Ok(())
    }

    fn eval_unary_expression(&mut self, node: ASTNode) -> Result<Option<Symbol>, String> {
        let symbol = match self.eval_node(node)? {
            Some(s) => s,
            None => return Ok(None),
        };

        let res = match symbol {
            Symbol::Number(num) => Some(Symbol::Number(-num)),
            _ => None,
        };

        Ok(res)
    }

    fn eval_binary_expression(&mut self, be: BinaryExpression) -> Result<Option<Symbol>, String> {
        let left_symbol = match self.eval_node(*be.left)? {
            Some(s) => s,
            None => return Ok(None),
        };

        if be.operator == TokenType::And && !left_symbol.is_truthy() {
            return Ok(Some(left_symbol));
        }

        if be.operator == TokenType::Or && left_symbol.is_truthy() {
            return Ok(Some(left_symbol));
        }

        let right_symbol = match self.eval_node(*be.right)? {
            Some(s) => s,
            None => return Ok(None),
        };

        let symbol_result =
            symbol::eval_binary_expression(&left_symbol, &be.operator, &right_symbol)?;
        Ok(Some(symbol_result))
    }
}
