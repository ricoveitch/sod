use super::ast::FunctionExpression;

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Number(f64),
    Boolean(bool),
    String(String),
    Function(FunctionExpression),
    Variable(String),
}

impl ToString for Symbol {
    fn to_string(&self) -> String {
        match self {
            Symbol::Number(n) => n.to_string(),
            Symbol::Boolean(b) => b.to_string(),
            Symbol::Function(f) => f.name.to_string(),
            Symbol::String(s) | Symbol::Variable(s) => s.to_string(),
        }
    }
}
