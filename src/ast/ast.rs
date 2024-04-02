use crate::lexer::token::TokenType;

#[derive(Debug, Clone)]
pub enum ASTNode {
    Program(Box<Vec<ASTNode>>),

    MemberExpression(MemberExpression),
    FunctionExpression(FunctionExpression),
    FunctionCall(FunctionCall),

    ReturnExpression(Box<ASTNode>),
    VariableExpression(VariableExpression),
    BinaryExpression(BinaryExpression),
    UnaryExpression(Box<ASTNode>),

    IfStatement(IfStatement),
    BlockStatement(BlockStatement),

    Number(f64),
    Boolean(bool),
    String(String),
    Identifier(String),
    List(Box<Vec<ASTNode>>),

    Command(Box<Vec<ASTNode>>),
}

#[derive(Debug, Clone)]
pub enum MemberExpressionKind {
    Index(ASTNode),
    Call(FunctionCall),
}

#[derive(Debug, Clone)]
pub struct MemberExpression {
    pub identifier: String,
    pub kind: Box<MemberExpressionKind>,
}

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Box<ASTNode>,
    pub operator: TokenType,
    pub right: Box<ASTNode>,
}

#[derive(Debug, Clone)]
pub struct VariableExpression {
    pub lhs: Box<ASTNode>,
    pub rhs: Box<ASTNode>,
}

#[derive(Debug, Clone)]
pub struct FunctionExpression {
    pub name: String,
    pub body: Box<ASTNode>,
    pub args: Vec<String>,
}

impl PartialEq for FunctionExpression {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<ASTNode>,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Box<ASTNode>,
    pub consequence: Box<ASTNode>,
    pub alternative: Option<Box<ASTNode>>,
}

#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub body: Box<Vec<ASTNode>>,
}
