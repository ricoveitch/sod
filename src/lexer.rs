use std::error::Error;

pub struct Lexer {
    src: Vec<u8>,
    cursor: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Asterisk,
    Carat,
    CloseParenthesis,
    EOF,
    Equals,
    Minus,
    OpenParenthesis,
    Plus,
    ForwardSlash,
    OpenBraces,
    CloseBraces,
    Comma,
    Newline,
    Integer(usize),
    Decimal(f64),
    Identifier(String),
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
        Ok((TokenType::Integer(num), bytes_read))
    }

    fn read_identifier(&self) -> (TokenType, usize) {
        let (bytes, bytes_read) = self.read_while(|b| b.is_ascii_alphabetic());

        let word = match String::from_utf8(bytes) {
            Ok(w) => w,
            Err(_) => panic!("invalid word"),
        };

        return (TokenType::Identifier(word), bytes_read);
    }

    fn skip_space(&mut self) {
        while let Some(byte) = self.peak_byte() {
            if byte != &b' ' {
                return;
            }
            self.cursor += 1;
        }
    }

    fn next(&mut self) -> (TokenType, usize) {
        self.skip_space();

        let byte = match self.peak_byte() {
            Some(b) => b,
            None => return (TokenType::EOF, 0),
        };

        match byte {
            b'+' => (TokenType::Plus, 1),
            b'-' => (TokenType::Minus, 1),
            b'*' => (TokenType::Asterisk, 1),
            b'^' => (TokenType::Carat, 1),
            b'/' => (TokenType::ForwardSlash, 1),
            b'(' => (TokenType::OpenParenthesis, 1),
            b')' => (TokenType::CloseParenthesis, 1),
            b'=' => (TokenType::Equals, 1),
            b'{' => (TokenType::OpenBraces, 1),
            b'}' => (TokenType::CloseBraces, 1),
            b',' => (TokenType::Comma, 1),
            b'\n' => (TokenType::Newline, 1),
            b if b.is_ascii_digit() => match self.read_digit() {
                Ok(r) => r,
                Err(e) => panic!("{}", e),
            },
            b if b.is_ascii_alphabetic() => self.read_identifier(),
            _ => panic!("invalid character: {}", *byte as char),
        }
    }

    pub fn next_token(&mut self) -> TokenType {
        let (token, bytes_read) = self.next();
        self.cursor += bytes_read;

        token
    }

    pub fn lookahead(&mut self, distance: usize) -> TokenType {
        let mut i = distance as i32;
        let mut total_bytes_read: usize = 0;

        loop {
            let (token, bytes_read) = self.next();
            self.cursor += bytes_read;
            total_bytes_read += bytes_read;

            match token {
                TokenType::EOF => {
                    self.cursor -= total_bytes_read;
                    return token;
                }
                _ => (),
            }

            i -= 1;
            if i < 0 {
                self.cursor -= total_bytes_read;
                return token;
            }
        }
    }
}
