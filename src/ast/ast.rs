use crate::lexer::TokenType;

#[derive(Debug, Clone)]
pub enum ASTNode {
    Number(f64),
    BinaryExpression(BinaryExpression),
    UnaryExpression(Box<ASTNode>),
    VariableExpression(VariableExpression),
    Variable(String),
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

impl BinaryExpression {
    pub fn new(left: ASTNode, operator: TokenType, right: ASTNode) -> BinaryExpression {
        BinaryExpression {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}
