use crate::lexer::TokenType;

use super::ast::{ASTNode, BinaryExpression};

pub fn visit(node: ASTNode) -> f64 {
    match node {
        ASTNode::Number(value) => return value,
        ASTNode::BinaryExpression(be) => visit_binary_expression(be),
        ASTNode::UnaryExpression(n) => visit_unary_expression(*n),
        ASTNode::Empty => 0.0,
    }
}

fn visit_unary_expression(node: ASTNode) -> f64 {
    -visit(node)
}

fn visit_binary_expression(be: BinaryExpression) -> f64 {
    let l = visit(*be.left);
    let r = visit(*be.right);

    match be.operator {
        TokenType::Plus => l + r,
        TokenType::Minus => l - r,
        TokenType::Asterisk => l * r,
        TokenType::Slash => l / r,
        TokenType::Carat => l.powf(r),
        _ => panic!(""),
    }
}
