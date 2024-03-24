use orca::lexer::Lexer;
use orca::lexer::TokenType;

fn assert_tokens(mut l: Lexer, expected: Vec<TokenType>, cmd: bool) {
    let mut next = || {
        if cmd {
            return l.next_cmd_token();
        } else {
            return l.next_token();
        };
    };

    for expect in expected {
        let t = next();
        assert_eq!(expect, t);
    }

    assert_eq!(TokenType::EOF, next());
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
            false,
        );

        assert_tokens(
            Lexer::new("1+2\n"),
            vec![
                TokenType::Integer(1),
                TokenType::Plus,
                TokenType::Integer(2),
                TokenType::Newline,
            ],
            false,
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
            false,
        );
    }

    #[test]
    fn strings() {
        assert_tokens(
            Lexer::new(r#"x="foo""#),
            vec![
                TokenType::Identifier("x".to_string()),
                TokenType::Equals,
                TokenType::String("foo".to_string()),
            ],
            false,
        );
    }

    #[test]
    fn bash() {
        assert_tokens(
            Lexer::new("ls -la >> foo.txt"),
            vec![
                TokenType::Identifier("ls".to_string()),
                TokenType::Whitespace,
                TokenType::Minus,
                TokenType::Identifier("la".to_string()),
                TokenType::Whitespace,
                TokenType::AppendOutput,
                TokenType::Whitespace,
                TokenType::Identifier("foo".to_string()),
                TokenType::Dot,
                TokenType::Identifier("txt".to_string()),
            ],
            true,
        );
    }
}
