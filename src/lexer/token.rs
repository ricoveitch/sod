#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Ampersand,
    And,
    AppendOutput,
    Asterisk,
    Carat,
    CloseBraces,
    CloseParen,
    Comma,
    Dot,
    DoubleEquals,
    EOF,
    Equals,
    ForwardSlash,
    Ge,
    GreaterThan,
    HereDocument,
    Le,
    LessThan,
    Minus,
    Newline,
    Not,
    NotEquals,
    OpenBraces,
    OpenParen,
    Or,
    Pipe,
    Plus,
    QuestionMark,
    SemiColon,
    SingleQuote,
    Tilde,
    Whitespace,
    LineComment,
    Integer(usize),
    Decimal(f64),
    String(StringToken),
    Identifier(String),
    EscapedIdentifier(String),
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
            TokenType::Ampersand => "&",
            TokenType::And => "&&",
            TokenType::AppendOutput => ">>",
            TokenType::Asterisk => "*",
            TokenType::Carat => "^",
            TokenType::CloseBraces => "}",
            TokenType::CloseParen => ")",
            TokenType::Comma => ",",
            TokenType::Dot => ".",
            TokenType::DoubleEquals => "==",
            TokenType::EOF => "EOF",
            TokenType::Equals => ".",
            TokenType::ForwardSlash => "/",
            TokenType::Ge => ">=",
            TokenType::GreaterThan => ">",
            TokenType::HereDocument => "<<",
            TokenType::Le => "<=",
            TokenType::LessThan => "<",
            TokenType::Minus => "-",
            TokenType::Newline => "\n",
            TokenType::Not => "!",
            TokenType::NotEquals => "!=",
            TokenType::OpenBraces => "{",
            TokenType::OpenParen => "(",
            TokenType::Or => "||",
            TokenType::Pipe => "|",
            TokenType::Plus => "+",
            TokenType::QuestionMark => "?",
            TokenType::SemiColon => ";",
            TokenType::SingleQuote => "'",
            TokenType::Tilde => "~",
            TokenType::Whitespace => " ",
            TokenType::LineComment => "",
            TokenType::EscapedIdentifier(s) => s.as_str(),
            TokenType::Integer(i) => return write!(f, "{}", i),
            TokenType::Decimal(d) => return write!(f, "{}", d),
            TokenType::Identifier(s) => return write!(f, "{}", s),
            TokenType::String(s) => return write!(f, "{}{}{}", s.quote, s.value, s.quote),
        };

        write!(f, "{}", s)
    }
}
