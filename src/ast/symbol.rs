use super::ast::FunctionExpression;

#[derive(Debug, Clone)]
pub enum Symbol {
    Number(f64),
    Function(FunctionExpression),
    Variable(String),
}
