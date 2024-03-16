use crate::lexer::TokenType;

pub fn is_comparative_operator(operator: &TokenType) -> bool {
    match operator {
        TokenType::And
        | TokenType::Or
        | TokenType::DoubleEquals
        | TokenType::GreaterThan
        | TokenType::LessThan
        | TokenType::GreaterThanOrEqualTo
        | TokenType::LessThanOrEqualTo => true,
        _ => false,
    }
}
