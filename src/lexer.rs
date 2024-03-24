use std::error::Error;

use crate::common::utils;

pub struct Lexer {
    src: Vec<u8>,
    cursor: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    And,
    AppendOutput,
    Asterisk,
    Carat,
    CloseBraces,
    CloseParenthesis,
    Comma,
    Dot,
    DoubleEquals,
    EOF,
    Equals,
    ForwardSlash,
    GreaterThan,
    GreaterThanOrEqualTo,
    HereDocument,
    LessThan,
    LessThanOrEqualTo,
    Minus,
    Newline,
    NotEquals,
    OpenBraces,
    OpenParenthesis,
    Or,
    Pipe,
    Plus,
    SingleQuote,
    Whitespace,
    Integer(usize),
    Decimal(f64),
    String(String),
    Identifier(String),
    EscapedIdentifier(String),
}

impl TokenType {
    pub fn is_end_line(&self) -> bool {
        match self {
            TokenType::EOF | TokenType::Newline => true,
            _ => false,
        }
    }
}

impl ToString for TokenType {
    fn to_string(&self) -> String {
        let s = match &self {
            TokenType::And => "&&",
            TokenType::AppendOutput => ">>",
            TokenType::Asterisk => "*",
            TokenType::Carat => "^",
            TokenType::CloseBraces => "}",
            TokenType::CloseParenthesis => ")",
            TokenType::Comma => ",",
            TokenType::Dot => ".",
            TokenType::DoubleEquals => "==",
            TokenType::EOF => "EOF",
            TokenType::Equals => ".",
            TokenType::ForwardSlash => "/",
            TokenType::GreaterThan => ">",
            TokenType::GreaterThanOrEqualTo => ">=",
            TokenType::HereDocument => "<<",
            TokenType::LessThan => "<",
            TokenType::LessThanOrEqualTo => "<=",
            TokenType::Minus => "-",
            TokenType::Newline => "\n",
            TokenType::NotEquals => "!=",
            TokenType::OpenBraces => "{",
            TokenType::OpenParenthesis => "(",
            TokenType::Or => "||",
            TokenType::Pipe => "|",
            TokenType::Plus => "+",
            TokenType::SingleQuote => "'",
            TokenType::Whitespace => " ",
            TokenType::Integer(i) => return i.to_string(),
            TokenType::Decimal(d) => return d.to_string(),
            TokenType::Identifier(s) => s.as_str(),
            TokenType::String(s) => return format!(r#""{}""#, s),
            TokenType::EscapedIdentifier(s) => s.as_str(),
        };

        s.to_string()
    }
}

fn is_whitespace(byte: u8) -> bool {
    match byte {
        b' ' | b'\t' | b'\r' => true,
        _ => false,
    }
}

impl Lexer {
    pub fn new(src: &str) -> Lexer {
        Lexer {
            src: src.as_bytes().to_owned(),
            cursor: 0,
        }
    }

    fn peak_byte(&self, distance: usize) -> Option<&u8> {
        self.src.get(self.cursor + distance)
    }

    fn peaked_byte_is(&self, target: &u8, distance: usize) -> bool {
        match self.peak_byte(distance) {
            Some(b) => b == target,
            None => false,
        }
    }

    fn read_while(&self, mut pred: impl FnMut(&u8) -> bool, offset: usize) -> (Vec<u8>, usize) {
        let mut bytes = vec![];
        for byte in self.src.iter().skip(self.cursor + offset) {
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
        let (bytes, bytes_read) = self.read_while(
            |b: &u8| {
                if b == &b'.' {
                    if seen_dot {
                        return false;
                    }
                    seen_dot = true;
                    return true;
                }

                b.is_ascii_digit()
            },
            0,
        );

        let s = String::from_utf8(bytes)?;

        if seen_dot {
            let dec = s.parse()?;
            return Ok((TokenType::Decimal(dec), bytes_read));
        }

        let num = s.parse()?;
        Ok((TokenType::Integer(num), bytes_read))
    }

    fn read_identifier(&self) -> (TokenType, usize) {
        let (bytes, bytes_read) = self.read_while(|b| b.is_ascii_alphabetic(), 0);

        (
            TokenType::Identifier(utils::bytes_to_string(bytes)),
            bytes_read,
        )
    }

    fn read_equals(&self) -> (TokenType, usize) {
        match self.peak_byte(1) {
            Some(b) if b == &b'=' => (TokenType::DoubleEquals, 2),
            _ => (TokenType::Equals, 1),
        }
    }

    fn read_not_equals(&self) -> (TokenType, usize) {
        match self.peak_byte(1) {
            Some(b) if b == &b'=' => (TokenType::NotEquals, 2),
            _ => panic!("invalid character '!'",),
        }
    }

    fn read_left_arrow(&self) -> (TokenType, usize) {
        let next = match self.peak_byte(1) {
            Some(b) => b,
            _ => return (TokenType::LessThan, 1),
        };

        match next {
            &b'=' => (TokenType::LessThanOrEqualTo, 2),
            &b'<' => (TokenType::HereDocument, 2),
            _ => (TokenType::LessThan, 1),
        }
    }

    fn read_right_arrow(&self) -> (TokenType, usize) {
        let next = match self.peak_byte(1) {
            Some(b) => b,
            _ => return (TokenType::GreaterThan, 1),
        };

        match next {
            &b'=' => (TokenType::GreaterThanOrEqualTo, 2),
            &b'>' => (TokenType::AppendOutput, 2),
            _ => (TokenType::GreaterThan, 1),
        }
    }

    fn read_and(&self) -> (TokenType, usize) {
        if !self.peaked_byte_is(&b'&', 1) {
            panic!("expected &&");
        }

        (TokenType::And, 2)
    }

    fn read_pipe(&self) -> (TokenType, usize) {
        match self.peak_byte(1) {
            Some(b) if b == &b'|' => (TokenType::Or, 2),
            _ => (TokenType::Pipe, 1),
        }
    }

    fn read_string(&self) -> (TokenType, usize) {
        let (s_bytes, s_bytes_read) = self.read_while(|b| *b != b'"', 1);

        (
            TokenType::String(utils::bytes_to_string(s_bytes)),
            s_bytes_read + 2,
        )
    }

    fn read_escaped_identifier(&self) -> (TokenType, usize) {
        let (bytes, bytes_read) = self.read_while(|b| b.is_ascii_alphanumeric(), 1);
        if bytes_read == 0 {
            panic!("expected a variable");
        }

        (
            TokenType::EscapedIdentifier(utils::bytes_to_string(bytes)),
            bytes_read + 1,
        )
    }

    fn read_whitespace(&self) -> (TokenType, usize) {
        let (_, bytes_read) = self.read_while(|b| is_whitespace(*b), 0);
        (TokenType::Whitespace, bytes_read)
    }

    fn next(&self) -> (TokenType, usize) {
        let byte = match self.peak_byte(0) {
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
            b'{' => (TokenType::OpenBraces, 1),
            b'}' => (TokenType::CloseBraces, 1),
            b',' => (TokenType::Comma, 1),
            b'\'' => (TokenType::SingleQuote, 1),
            b'.' => (TokenType::Dot, 1),
            b'\n' => (TokenType::Newline, 1),
            b'|' => self.read_pipe(),
            b'&' => self.read_and(),
            b'=' => self.read_equals(),
            b'!' => self.read_not_equals(),
            b'>' => self.read_right_arrow(),
            b'<' => self.read_left_arrow(),
            b'"' => self.read_string(),
            b'$' => self.read_escaped_identifier(),
            b if is_whitespace(*b) => self.read_whitespace(),
            b if b.is_ascii_digit() => match self.read_digit() {
                Ok(r) => r,
                Err(e) => panic!("{}", e),
            },
            b if b.is_ascii_alphabetic() => self.read_identifier(),
            _ => panic!("invalid character: {}", *byte as char),
        }
    }

    pub fn next_token(&mut self) -> TokenType {
        loop {
            let (token, bytes_read) = self.next();
            self.cursor += bytes_read;

            if token != TokenType::Whitespace {
                return token;
            }
        }
    }

    pub fn next_cmd_token(&mut self) -> TokenType {
        let (token, bytes_read) = self.next();
        self.cursor += bytes_read;
        token
    }

    pub fn lookahead(&mut self, distance: usize) -> TokenType {
        let mut i = distance as u32;
        let mut total_bytes_read: usize = 0;

        loop {
            let (token, bytes_read) = self.next();
            self.cursor += bytes_read;
            total_bytes_read += bytes_read;

            if token == TokenType::Whitespace {
                continue;
            }

            i -= 1;
            if i <= 0 || token == TokenType::EOF {
                self.cursor -= total_bytes_read;
                return token;
            }
        }
    }
}
