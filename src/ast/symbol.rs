use super::ast::FunctionExpression;

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Number(f64),
    Boolean(bool),
    String(String),
    Function(FunctionExpression),
    Variable(String),
}
