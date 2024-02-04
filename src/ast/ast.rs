use crate::lexer::TokenType;

#[derive(Debug)]
pub enum ASTNode {
    Number(f64),
    BinaryExpression(BinaryExpression),
    Empty,
}

#[derive(Debug)]
pub struct BinaryExpression {
    pub left: Box<ASTNode>,
    pub operator: TokenType,
    pub right: Box<ASTNode>,
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

pub struct Empty {}
