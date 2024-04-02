use super::ast::FunctionExpression;
use crate::lexer::token::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Number(f64),
    Boolean(bool),
    String(String),
    Function(FunctionExpression),
    Variable(String),
    List(List),
}

#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub items: Vec<Symbol>,
}

impl List {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn pop(&mut self) -> Option<Symbol> {
        self.items.pop()
    }

    pub fn push(&mut self, item: Symbol) -> usize {
        self.items.push(item);
        self.len()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Symbol> {
        self.items.get_mut(index)
    }

    pub fn get(&self, index: usize) -> Option<&Symbol> {
        self.items.get(index)
    }

    pub fn call(&mut self, fname: &str, args: Vec<Symbol>) -> Option<Symbol> {
        match fname {
            "len" => Some(Symbol::Number(self.len() as f64)),
            "pop" => self.pop(),
            "push" => {
                if args.len() != 1 {
                    panic!("incorrect number of arguments to push")
                }
                let symbol = args.get(0).unwrap().to_owned();
                return Some(Symbol::Number(self.push(symbol) as f64));
            }
            _ => panic!("list has no member '{}'", fname),
        }
    }
}

fn compare_literal<T>(left: T, operator: &TokenType, right: T) -> bool
where
    T: std::cmp::PartialEq + std::cmp::PartialOrd + std::fmt::Display,
{
    match operator {
        TokenType::GreaterThan => left > right,
        TokenType::LessThan => left < right,
        TokenType::Ge => left >= right,
        TokenType::Le => left <= right,
        _ => panic!(
            "{} {} {}: unable to compare booleans",
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
            Symbol::Function(f) => format!("func {}", f.name),
            Symbol::String(s) | Symbol::Variable(s) => s.to_string(),
            Symbol::List(list) => {
                let items: Vec<String> = list.items.iter().map(|f| f.to_string()).collect();
                format!("{:?}", items)
            }
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
            Symbol::List(_) => true,
        }
    }
}
