use orca::lexer::Lexer;
use orca::lexer::TokenType;

fn assert_tokens(mut l: Lexer, expected: Vec<TokenType>) {
    for expect in expected {
        let t = l.next_token();
        assert_eq!(t, expect);
    }

    assert_eq!(l.next_token(), TokenType::EOF);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn simple() {
        assert_tokens(
            Lexer::new("1+ 2 -1.2"),
            vec![
                TokenType::Integer(1),
                TokenType::Plus,
                TokenType::Integer(2),
                TokenType::Minus,
                TokenType::Decimal(1.2),
            ],
        );
    }

    #[test]
    fn assignment() {
        assert_tokens(
            Lexer::new("foo = 1"),
            vec![
                TokenType::Identifier("foo".to_string()),
                TokenType::Equals,
                TokenType::Integer(1),
            ],
        );
    }
}
