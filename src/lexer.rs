use std::error::Error;

pub struct Lexer {
    src: Vec<u8>,
    cursor: usize,
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Integer(usize),
    Decimal(f64),
    Identifier(String),
    Plus,
    Minus,
    Asterisk,
    Carat,
    Slash,
    OpenParenthesis,
    CloseParenthesis,
    Equals,
    EOF,
}

impl Lexer {
    pub fn new(src: &str) -> Lexer {
        Lexer {
            src: src.as_bytes().to_owned(),
            cursor: 0,
        }
    }

    fn peak_byte(&self) -> Option<&u8> {
        self.src.get(self.cursor)
    }

    fn read_while(&self, mut pred: impl FnMut(&u8) -> bool) -> (Vec<u8>, usize) {
        let mut bytes = vec![];
        for byte in self.src.iter().skip(self.cursor) {
            if pred(byte) {
                bytes.push(byte.clone())
            } else {
                break;
            }
        }
        let bytes_read = bytes.len();
        (bytes, bytes_read)
    }

    fn read_digit(&self) -> Result<(TokenType, usize), Box<dyn Error>> {
        let mut seen_dot = false;
        let (bytes, bytes_read) = self.read_while(|b: &u8| {
            if b == &b'.' {
                if seen_dot {
                    return false;
                }
                seen_dot = true;
                return true;
            }

            return b.is_ascii_digit();
        });

        let s = String::from_utf8(bytes)?;

        if seen_dot {
            let dec = s.parse()?;
            return Ok((TokenType::Decimal(dec), bytes_read));
        }

        let num = s.parse()?;
        return Ok((TokenType::Integer(num), bytes_read));
    }

    fn read_word(&self) -> (TokenType, usize) {
        let (bytes, bytes_read) = self.read_while(|b| b.is_ascii_alphabetic());

        let word = match String::from_utf8(bytes) {
            Ok(w) => w,
            Err(_) => panic!("invalid word"),
        };

        return (TokenType::Identifier(word), bytes_read);
    }

    fn skip_whitespace(&mut self) {
        while let Some(byte) = self.peak_byte() {
            if !byte.is_ascii_whitespace() {
                return;
            }
            self.cursor += 1;
        }
    }

    pub fn next_token(&mut self) -> TokenType {
        self.skip_whitespace();

        let byte = match self.peak_byte() {
            Some(b) => b,
            None => return TokenType::EOF,
        };

        let (token, bytes_read) = match byte {
            b'+' => (TokenType::Plus, 1),
            b'-' => (TokenType::Minus, 1),
            b'*' => (TokenType::Asterisk, 1),
            b'^' => (TokenType::Carat, 1),
            b'/' => (TokenType::Slash, 1),
            b'(' => (TokenType::OpenParenthesis, 1),
            b')' => (TokenType::CloseParenthesis, 1),
            b'=' => (TokenType::Equals, 1),
            b if b.is_ascii_digit() => match self.read_digit() {
                Ok(r) => r,
                Err(e) => panic!("{}", e),
            },
            b if b.is_ascii_alphabetic() => self.read_word(),
            _ => panic!("unknown character"),
        };

        self.cursor += bytes_read;

        token
    }
}
