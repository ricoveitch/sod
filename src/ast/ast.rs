use crate::lexer::TokenType;

#[derive(Debug, Clone)]
pub enum ASTNode {
    Program(Box<Vec<ASTNode>>),
    FunctionExpression(FunctionExpression),
    FunctionCall(String),
    ReturnExpression(Box<ASTNode>),
    VariableExpression(VariableExpression),
    BinaryExpression(BinaryExpression),
    UnaryExpression(Box<ASTNode>),
    Variable(String),
    Number(f64),
}

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Box<ASTNode>,
    pub operator: TokenType,
    pub right: Box<ASTNode>,
}

#[derive(Debug, Clone)]
pub struct VariableExpression {
    pub name: String,
    pub value: Box<ASTNode>,
}

#[derive(Debug, Clone)]
pub struct FunctionExpression {
    pub name: String,
    pub body: Box<Vec<ASTNode>>,
}
