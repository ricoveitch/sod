use super::ast::FunctionExpression;
use crate::lexer::token::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Number(f64),
    Boolean(bool),
    String(String),
    Function(FunctionExpression),
    Variable(String),
}

fn compare_literal<T>(left: T, operator: &TokenType, right: T) -> bool
where
    T: std::cmp::PartialEq + std::cmp::PartialOrd + std::fmt::Debug,
{
    match operator {
        TokenType::GreaterThan => left > right,
        TokenType::LessThan => left < right,
        TokenType::Ge => left >= right,
        TokenType::Le => left <= right,
        _ => panic!(
            "{:?} {} {:?}: unable to compare booleans",
            left, operator, right
        ),
    }
}

fn compare_relational(left: &Symbol, op: &TokenType, right: &Symbol) -> bool {
    match (left, right) {
        (Symbol::Number(lv), Symbol::Number(rv)) => compare_literal(lv, op, rv),
        (Symbol::Boolean(lv), Symbol::Boolean(rv)) => compare_literal(lv, op, rv),
        (Symbol::String(lv), Symbol::String(rv)) => compare_literal(lv, op, rv),
        _ => panic!("type mismatch: {} > {}", left, right),
    }
}

pub fn eval_binary_expression(left: &Symbol, operator: &TokenType, right: &Symbol) -> Symbol {
    match operator {
        TokenType::Plus => left + right,
        TokenType::Minus => left - right,
        TokenType::Asterisk => left * right,
        TokenType::ForwardSlash => left / right,
        TokenType::Carat => match (left, right) {
            (Symbol::Number(ln), Symbol::Number(rn)) => Symbol::Number(ln.powf(*rn)),
            _ => panic!("can't raise the power of non-number ({}^{})", left, right),
        },
        TokenType::DoubleEquals => Symbol::Boolean(left == right),
        TokenType::NotEquals => Symbol::Boolean(left != right),
        TokenType::And => right.clone(),
        TokenType::Or => {
            if left.is_truthy() {
                left.clone()
            } else {
                right.clone()
            }
        }
        TokenType::GreaterThan | TokenType::LessThan | TokenType::Ge | TokenType::Le => {
            Symbol::Boolean(compare_relational(left, operator, right))
        }
        _ => panic!("unsupported operator {}", operator),
    }
}

impl std::ops::Add for &Symbol {
    type Output = Symbol;

    fn add(self, rhs: Self) -> Symbol {
        match (self, rhs) {
            (Symbol::Number(lv), Symbol::Number(rv)) => Symbol::Number(lv + rv),
            (Symbol::String(lv), Symbol::String(rv)) => Symbol::String(format!("{}{}", lv, rv)),
            _ => panic!("unsupported operand type for {} + {}", self, rhs),
        }
    }
}

impl std::ops::Sub for &Symbol {
    type Output = Symbol;

    fn sub(self, rhs: Self) -> Symbol {
        match (self, rhs) {
            (Symbol::Number(lv), Symbol::Number(rv)) => Symbol::Number(lv - rv),
            _ => panic!("unsupported operand type for {} - {}", self, rhs),
        }
    }
}

impl std::ops::Mul for &Symbol {
    type Output = Symbol;

    fn mul(self, rhs: Self) -> Symbol {
        match (self, rhs) {
            (Symbol::Number(lv), Symbol::Number(rv)) => Symbol::Number(lv * rv),
            _ => panic!("unsupported operand type for {} * {}", self, rhs),
        }
    }
}

impl std::ops::Div for &Symbol {
    type Output = Symbol;

    fn div(self, rhs: Self) -> Symbol {
        match (self, rhs) {
            (Symbol::Number(lv), Symbol::Number(rv)) => Symbol::Number(lv / rv),
            _ => panic!("unsupported operand type for {} / {}", self, rhs),
        }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Symbol::Number(n) => n.to_string(),
            Symbol::Boolean(b) => b.to_string(),
            Symbol::Function(f) => f.name.to_string(),
            Symbol::String(s) | Symbol::Variable(s) => s.to_string(),
        };

        write!(f, "{}", s)
    }
}

impl Symbol {
    pub fn is_truthy(&self) -> bool {
        match self {
            Symbol::Number(n) => *n != 0.0,
            Symbol::Boolean(b) => *b,
            Symbol::Function(_) => true,
            Symbol::String(s) | Symbol::Variable(s) => s.len() > 0,
        }
    }
}
