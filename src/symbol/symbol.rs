use std::collections::HashMap;

use crate::ast::ast::FunctionStatement;
use crate::lexer::token::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Number(f64),
    Boolean(bool),
    String(StringSymbol),
    List(List),
    Range(Range),
    None,
    Function(Box<FunctionStatement>),
    Object(Object),
}

#[macro_export]
macro_rules! new_string_symbol {
    ($v:expr) => {
        $crate::symbol::symbol::Symbol::String($crate::symbol::symbol::StringSymbol::new($v))
    };
}

pub fn get_global_vars(argv: Vec<String>) -> Vec<(&'static str, Symbol)> {
    // change process to script?
    vec![(
        "process",
        Symbol::Object(Object::from(vec![(
            "argv",
            Symbol::List(List::from(
                argv.iter()
                    .map(|arg| new_string_symbol!(arg.to_string()))
                    .collect(),
            )),
        )])),
    )]
}

#[derive(PartialEq, Debug, Clone)]
pub struct Object {
    mapping: HashMap<String, Symbol>,
}

impl Object {
    pub fn from(items: Vec<(&str, Symbol)>) -> Self {
        let mut mapping = HashMap::new();
        for (key, value) in items {
            mapping.insert(key.to_string(), value);
        }
        Self { mapping }
    }

    pub fn get(&self, key: &str) -> &Symbol {
        self.mapping.get(key).unwrap_or(&Symbol::None)
    }

    pub fn get_mut(&mut self, key: &str) -> &mut Symbol {
        self.mapping.get_mut(key).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Range {
    pub start: i32,
    pub end: i32,
    pub increment: i32,
    ticker: i32,
}

impl Range {
    pub fn new(start: i32, end: i32, increment: Option<i32>) -> Self {
        Self {
            start,
            end,
            increment: increment.unwrap_or(1),
            ticker: start,
        }
    }

    fn next(&mut self) -> Option<Symbol> {
        if self.increment > 0 && self.ticker >= self.end {
            return None;
        } else if self.increment < 0 && self.ticker <= self.end {
            return None;
        }

        let result = Symbol::Number(self.ticker as f64);
        self.ticker += self.increment;
        Some(result)
    }
}

impl Iterator for Range {
    type Item = Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl PartialEq for Range {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct StringSymbol {
    value: String,
}

impl StringSymbol {
    pub fn new(s: String) -> Self {
        Self { value: s }
    }

    pub fn get(&self, index: usize) -> Symbol {
        match self.value.chars().nth(index) {
            Some(c) => new_string_symbol!(c.to_string()),
            None => panic!("string index out of range"),
        }
    }

    pub fn len(&self) -> Symbol {
        Symbol::Number(self.value.len() as f64)
    }

    pub fn insert(&mut self, args: Vec<Symbol>) {
        if args.len() != 2 {
            panic!("expected 2 arguments to insert, found {}", args.len())
        }

        let index = match args.get(0).unwrap().to_owned() {
            Symbol::Number(index) => index as usize,
            _ => panic!("string indexes must be of type number"),
        };

        if index > self.value.len() {
            panic!("string insert index out of range");
        }

        let string = match args.get(1).unwrap() {
            Symbol::String(s) => &s.value,
            _ => panic!("can only insert string into a string"),
        };

        self.value.insert_str(index, string.as_str());
    }

    pub fn remove(&mut self, args: Vec<Symbol>) -> Symbol {
        if args.len() != 1 {
            panic!("incorrect number of arguments to remove")
        }

        let index = match args.get(0).unwrap().to_owned() {
            Symbol::Number(index) => index as usize,
            _ => panic!("string indexes must be of type number"),
        };

        if index > self.value.len() {
            panic!("string remove index out of range");
        }

        let removed = self.value.remove(index);
        new_string_symbol!(removed.to_string())
    }

    pub fn pop(&mut self) -> Option<Symbol> {
        if let Some(popped) = self.value.pop() {
            return Some(new_string_symbol!(popped.to_string()));
        }

        return None;
    }

    pub fn push(&mut self, args: Vec<Symbol>) -> Symbol {
        if args.len() != 1 {
            panic!("incorrect number of arguments to push")
        }

        let symbol = match args.get(0).unwrap() {
            Symbol::String(ss) => &ss.value,
            _ => panic!("can only add a string to a string"),
        };

        self.value.push_str(symbol);
        self.len()
    }

    fn trim(&mut self) -> Symbol {
        let trimmed = self.value.trim();
        new_string_symbol!(trimmed.to_string())
    }

    pub fn call(&mut self, fname: &str, args: Vec<Symbol>) -> Option<Symbol> {
        match fname {
            "insert" => {
                self.insert(args);
                None
            }
            "remove" => Some(self.remove(args)),
            "pop" => self.pop(),
            "len" => Some(self.len()),
            "push" => Some(self.push(args)),
            "trim" => Some(self.trim()),
            _ => panic!("string has no member '{}'", fname),
        }
    }
}

pub struct StringSymbolIterator {
    value: String,
    index: usize,
}

impl Iterator for StringSymbolIterator {
    type Item = Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.value.len() {
            let next = new_string_symbol!(self.value.chars().nth(self.index).unwrap().to_string());
            self.index += 1;
            Some(next)
        } else {
            None
        }
    }
}

impl IntoIterator for StringSymbol {
    type Item = Symbol;
    type IntoIter = StringSymbolIterator;

    fn into_iter(self) -> Self::IntoIter {
        StringSymbolIterator {
            value: self.value,
            index: 0,
        }
    }
}

impl PartialEq for StringSymbol {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub items: Vec<Symbol>,
}

impl List {
    pub fn from(items: Vec<Symbol>) -> Self {
        Self { items }
    }

    pub fn len(&self) -> Symbol {
        Symbol::Number(self.items.len() as f64)
    }

    pub fn pop(&mut self) -> Option<Symbol> {
        self.items.pop()
    }

    pub fn push(&mut self, args: Vec<Symbol>) -> Symbol {
        if args.len() != 1 {
            panic!("incorrect number of arguments to push")
        }

        let symbol = args.get(0).unwrap().to_owned();
        self.items.push(symbol);
        self.len()
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Symbol {
        match self.items.get_mut(index) {
            Some(s) => s,
            None => panic!("list index out of range"),
        }
    }

    pub fn get(&self, index: usize) -> &Symbol {
        match self.items.get(index) {
            Some(s) => s,
            None => panic!("list index out of range"),
        }
    }

    pub fn remove(&mut self, args: Vec<Symbol>) -> Symbol {
        if args.len() != 1 {
            panic!("incorrect number of arguments to remove")
        }

        let index = match args.get(0).unwrap().to_owned() {
            Symbol::Number(index) => index as usize,
            _ => panic!("list indexes must be of type number"),
        };

        if index > self.items.len() {
            panic!("list remove index out of range");
        }

        self.items.remove(index)
    }

    pub fn insert(&mut self, args: Vec<Symbol>) {
        if args.len() != 2 {
            panic!("expected 2 arguments to insert, found {}", args.len())
        }

        let index = match args.get(0).unwrap().to_owned() {
            Symbol::Number(index) => index as usize,
            _ => panic!("list indexes must be of type number"),
        };

        if index > self.items.len() {
            panic!("list insert index out of range");
        }

        let symbol = args.get(1).unwrap().to_owned();
        self.items.insert(index, symbol);
    }

    pub fn call(&mut self, fname: &str, args: Vec<Symbol>) -> Option<Symbol> {
        match fname {
            "len" => Some(self.len()),
            "pop" => self.pop(),
            "push" => Some(self.push(args)),
            "remove" => Some(self.remove(args)),
            "insert" => {
                self.insert(args);
                None
            }
            _ => panic!("list has no member '{}'", fname),
        }
    }
}

fn compare_literal<T>(left: &T, operator: &TokenType, right: &T) -> bool
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
        (Symbol::String(lv), Symbol::String(rv)) => compare_literal(&lv.value, op, &rv.value),
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
            (Symbol::String(lv), Symbol::String(rv)) => {
                let value = format!("{}{}", lv.value, rv.value);
                new_string_symbol!(value)
            }
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
            Symbol::String(s) => s.value.to_string(),
            Symbol::None => "none".to_string(),
            Symbol::List(list) => {
                let items: Vec<String> = list.items.iter().map(|f| f.to_string()).collect();
                format!("{:?}", items)
            }
            Symbol::Range(range) => format!("{}..{}..{}", range.start, range.end, range.increment),
            Symbol::Object(obj) => format!("{:?}", obj.mapping),
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
            Symbol::String(s) => s.value.len() > 0,
            Symbol::List(_) => true,
            Symbol::None => false,
            Symbol::Range(_) => true,
            Symbol::Object(_) => true,
        }
    }

    pub fn kind(&self) -> String {
        let s = match self {
            Symbol::Number(_) => "number",
            Symbol::Boolean(_) => "boolean",
            Symbol::Function(_) => "function",
            Symbol::String(_) => "string",
            Symbol::List(_) => "list",
            Symbol::None => "none",
            Symbol::Range(_) => "range",
            Symbol::Object(_) => "object",
        };

        s.to_string()
    }
}
