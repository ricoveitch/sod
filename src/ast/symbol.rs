use super::ast::FunctionExpression;

#[derive(Debug, Clone)]
pub enum Symbol {
    Number(f64),
    Boolean(bool),
    Function(FunctionExpression),
    Variable(String),
}
