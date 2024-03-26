use crate::lexer::token::TokenType;

pub fn is_comparative_operator(operator: &TokenType) -> bool {
    match operator {
        TokenType::And
        | TokenType::Or
        | TokenType::DoubleEquals
        | TokenType::NotEquals
        | TokenType::GreaterThan
        | TokenType::LessThan
        | TokenType::Ge
        | TokenType::Le => true,
        _ => false,
    }
}
