#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    And,
    Asterisk,
    Carat,
    CloseBraces,
    CloseParen,
    Comma,
    BackSlash,
    Dot,
    DoubleEquals,
    EOF,
    Equals,
    ForwardSlash,
    Ge,
    GreaterThan,
    Le,
    LessThan,
    OpenSqBracket,
    CloseSqBracket,
    Minus,
    Newline,
    Not,
    NotEquals,
    OpenBraces,
    OpenParen,
    Or,
    Plus,
    SemiColon,
    SingleQuote,
    Whitespace,
    Underscore,
    LineComment,
    Integer(usize),
    Decimal(f64),
    String(StringToken),
    Identifier(String),
    EscapedIdentifier(String),
    CatchAll(String),
}

#[derive(Debug, Clone)]
pub struct StringToken {
    pub value: String,
    pub quote: char,
}

impl PartialEq for StringToken {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl ToString for StringToken {
    fn to_string(&self) -> String {
        format!("{}{}{}", self.quote, self.value, self.quote)
    }
}

impl TokenType {
    pub fn is_end_line(&self) -> bool {
        match self {
            TokenType::EOF | TokenType::Newline => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match &self {
            TokenType::And => "&&",
            TokenType::Asterisk => "*",
            TokenType::Carat => "^",
            TokenType::CloseBraces => "}",
            TokenType::CloseParen => ")",
            TokenType::Comma => ",",
            TokenType::BackSlash => "\\",
            TokenType::Dot => ".",
            TokenType::DoubleEquals => "==",
            TokenType::EOF => "EOF",
            TokenType::Equals => "=",
            TokenType::ForwardSlash => "/",
            TokenType::Ge => ">=",
            TokenType::GreaterThan => ">",
            TokenType::OpenSqBracket => "[",
            TokenType::CloseSqBracket => "]",
            TokenType::Le => "<=",
            TokenType::LessThan => "<",
            TokenType::Minus => "-",
            TokenType::Newline => "\n",
            TokenType::Not => "!",
            TokenType::NotEquals => "!=",
            TokenType::OpenBraces => "{",
            TokenType::OpenParen => "(",
            TokenType::Or => "||",
            TokenType::Plus => "+",
            TokenType::SemiColon => ";",
            TokenType::SingleQuote => "'",
            TokenType::Whitespace => " ",
            TokenType::Underscore => "_",
            TokenType::LineComment => "",
            TokenType::EscapedIdentifier(s) => s.as_str(),
            TokenType::Integer(i) => return write!(f, "{}", i),
            TokenType::Decimal(d) => return write!(f, "{}", d),
            TokenType::Identifier(s) => return write!(f, "{}", s),
            TokenType::String(s) => return write!(f, "{}{}{}", s.quote, s.value, s.quote),
            TokenType::CatchAll(s) => s.as_str(),
        };

        write!(f, "{}", s)
    }
}
