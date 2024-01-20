use orca::lexer::Lexer;
use orca::lexer::TokenType;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn simple() {
        let mut l = Lexer::new("1+ 2 -1.2");
        let expected = vec![
            TokenType::Integer(1),
            TokenType::Plus,
            TokenType::Integer(2),
            TokenType::Minus,
            TokenType::Decimal(1.2),
        ];

        for expect in expected {
            let t = l.next_token();
            assert_eq!(t, expect);
        }

        assert_eq!(l.next_token(), TokenType::EOF);
    }

    #[test]
    fn assignment() {
        let mut l = Lexer::new("foo = 1");
        let expected = vec![
            TokenType::Identifier("foo".to_string()),
            TokenType::Equals,
            TokenType::Integer(1),
        ];

        for expect in expected {
            let t = l.next_token();
            assert_eq!(t, expect);
        }

        assert_eq!(l.next_token(), TokenType::EOF);
    }
}
