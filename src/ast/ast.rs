use crate::lexer::TokenType;

use super::symbol::Symbol;

#[derive(Debug, Clone)]
pub enum ASTNode {
    Program(Box<Vec<ASTNode>>),

    FunctionExpression(FunctionExpression),
    FunctionCall(FunctionCall),

    ReturnExpression(Box<ASTNode>),
    VariableExpression(VariableExpression),
    BinaryExpression(BinaryExpression),
    UnaryExpression(Box<ASTNode>),

    IfStatement(IfStatement),

    Variable(String),
    Number(f64),
    Boolean(bool),
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
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Symbol>,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Box<ASTNode>,
    pub consequence: Box<Vec<ASTNode>>,
    pub alternative: Option<Box<ASTNode>>,
}
