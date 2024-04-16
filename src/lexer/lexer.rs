use std::error::Error;

use super::token::{StringToken, TokenType};
use crate::common::utils;

pub struct Lexer {
    src: Vec<u8>,
    cursor: usize,
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
        let read = self.read_while(
            |b| {
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

        // trim trailing dots
        let (bytes, bytes_read) = {
            let (mut bytes, _) = read;
            for i in (0..bytes.len()).rev() {
                if bytes[i] == b'.' {
                    bytes.pop();
                } else {
                    break;
                }
            }
            let len = bytes.len();
            (bytes, len)
        };

        let s = String::from_utf8(bytes)?;

        if seen_dot {
            let dec = s.parse()?;
            return Ok((TokenType::Decimal(dec), bytes_read));
        }

        let num = s.parse()?;
        Ok((TokenType::Integer(num), bytes_read))
    }

    fn read_identifier(&self) -> (TokenType, usize) {
        let (bytes, bytes_read) = self.read_while(|b| b.is_ascii_alphanumeric() || *b == b'_', 0);

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
            _ => (TokenType::NotEquals, 1),
        }
    }

    fn read_left_arrow(&self) -> (TokenType, usize) {
        let next = match self.peak_byte(1) {
            Some(b) => b,
            _ => return (TokenType::LessThan, 1),
        };

        match next {
            &b'=' => (TokenType::Le, 2),
            _ => (TokenType::LessThan, 1),
        }
    }

    fn read_right_arrow(&self) -> (TokenType, usize) {
        let next = match self.peak_byte(1) {
            Some(b) => b,
            _ => return (TokenType::GreaterThan, 1),
        };

        match next {
            &b'=' => (TokenType::Ge, 2),
            _ => (TokenType::GreaterThan, 1),
        }
    }

    fn read_and(&self) -> (TokenType, usize) {
        match self.peak_byte(1) {
            Some(b) if b == &b'&' => (TokenType::And, 2),
            _ => self.read_catch_all(b'&'),
        }
    }

    fn read_pipe(&self) -> (TokenType, usize) {
        match self.peak_byte(1) {
            Some(b) if b == &b'|' => (TokenType::Or, 2),
            _ => self.read_catch_all(b'|'),
        }
    }

    fn read_string(&self, term: u8) -> (TokenType, usize) {
        let (s_bytes, s_bytes_read) = self.read_while(|b| *b != term, 1);
        let s = utils::bytes_to_string(s_bytes);

        (
            TokenType::String(StringToken {
                value: s,
                quote: term as char,
            }),
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

    fn read_line_comment(&self) -> (TokenType, usize) {
        let (_, bytes_read) = self.read_while(|b| *b != b'\n', 0);
        (TokenType::LineComment, bytes_read)
    }

    fn read_whitespace(&self) -> (TokenType, usize) {
        let (_, bytes_read) = self.read_while(|b| is_whitespace(*b), 0);
        (TokenType::Whitespace, bytes_read)
    }

    fn read_catch_all(&self, byte: u8) -> (TokenType, usize) {
        let s = match String::from_utf8(vec![byte]) {
            Ok(s) => s,
            Err(_) => panic!("invalid character {}", byte as char),
        };

        (TokenType::CatchAll(s), 1)
    }

    fn peak(&self) -> (TokenType, usize) {
        let byte = match self.peak_byte(0) {
            Some(b) => b,
            None => return (TokenType::EOF, 0),
        };

        match byte {
            b'-' => (TokenType::Minus, 1),
            b',' => (TokenType::Comma, 1),
            b';' => (TokenType::SemiColon, 1),
            b'.' => (TokenType::Dot, 1),
            b'(' => (TokenType::OpenParen, 1),
            b')' => (TokenType::CloseParen, 1),
            b'{' => (TokenType::OpenBraces, 1),
            b'}' => (TokenType::CloseBraces, 1),
            b'*' => (TokenType::Asterisk, 1),
            b'/' => (TokenType::ForwardSlash, 1),
            b'\n' => (TokenType::Newline, 1),
            b'^' => (TokenType::Carat, 1),
            b'+' => (TokenType::Plus, 1),
            b'[' => (TokenType::OpenSqBracket, 1),
            b']' => (TokenType::CloseSqBracket, 1),
            b'\\' => (TokenType::BackSlash, 1),
            b'|' => self.read_pipe(),
            b'&' => self.read_and(),
            b'=' => self.read_equals(),
            b'!' => self.read_not_equals(),
            b'>' => self.read_right_arrow(),
            b'<' => self.read_left_arrow(),
            b'#' => self.read_line_comment(),
            b if *b == b'"' || *b == b'\'' => self.read_string(*b),
            b'$' => self.read_escaped_identifier(),
            b if is_whitespace(*b) => self.read_whitespace(),
            b if b.is_ascii_digit() => match self.read_digit() {
                Ok(r) => r,
                Err(e) => panic!("{}", e),
            },
            b if b.is_ascii_alphabetic() => self.read_identifier(),
            _ => self.read_catch_all(*byte),
        }
    }

    fn next(&mut self) -> TokenType {
        loop {
            let (token, bytes_read) = self.peak();
            self.cursor += bytes_read;
            if token != TokenType::LineComment {
                return token;
            }
        }
    }

    pub fn next_token(&mut self) -> TokenType {
        loop {
            let token = self.next();
            if token != TokenType::Whitespace {
                return token;
            }
        }
    }

    pub fn next_cmd_token(&mut self) -> TokenType {
        self.next()
    }

    pub fn lookahead(&mut self, distance: usize) -> TokenType {
        let mut i = distance as u32;
        let cursor_snapshot = self.cursor;

        loop {
            let token = self.next();
            if token == TokenType::Whitespace {
                continue;
            }

            i -= 1;
            if i <= 0 || token == TokenType::EOF {
                self.cursor = cursor_snapshot;
                return token;
            }
        }
    }
}
