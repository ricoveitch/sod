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

    pub fn get(&self, index: usize) -> Result<Symbol, String> {
        match self.value.chars().nth(index) {
            Some(c) => Ok(new_string_symbol!(c.to_string())),
            None => return Err(format!("string index out of range")),
        }
    }

    pub fn len(&self) -> Symbol {
        Symbol::Number(self.value.len() as f64)
    }

    pub fn insert(&mut self, args: Vec<Symbol>) -> Result<(), String> {
        if args.len() != 2 {
            return Err(format!(
                "expected 2 arguments to insert, found {}",
                args.len()
            ));
        }

        let index = match args.get(0).unwrap().to_owned() {
            Symbol::Number(index) => index as usize,
            _ => return Err(format!("string indexes must be of type number")),
        };

        if index > self.value.len() {
            return Err(format!("string insert index out of range"));
        }

        let string = match args.get(1).unwrap() {
            Symbol::String(s) => &s.value,
            _ => return Err(format!("can only insert string into a string")),
        };

        self.value.insert_str(index, string.as_str());

        Ok(())
    }

    pub fn remove(&mut self, args: Vec<Symbol>) -> Result<Symbol, String> {
        if args.len() != 1 {
            return Err(format!("incorrect number of arguments to remove"));
        }

        let index = match args.get(0).unwrap().to_owned() {
            Symbol::Number(index) => index as usize,
            _ => return Err(format!("string indexes must be of type number")),
        };

        if index > self.value.len() {
            return Err(format!("string remove index out of range"));
        }

        let removed = self.value.remove(index);
        Ok(new_string_symbol!(removed.to_string()))
    }

    pub fn pop(&mut self) -> Option<Symbol> {
        if let Some(popped) = self.value.pop() {
            return Some(new_string_symbol!(popped.to_string()));
        }

        return None;
    }

    pub fn push(&mut self, args: Vec<Symbol>) -> Result<Symbol, String> {
        if args.len() != 1 {
            return Err(format!("incorrect number of arguments to push"));
        }

        let symbol = match args.get(0).unwrap() {
            Symbol::String(ss) => &ss.value,
            _ => return Err(format!("can only add a string to a string")),
        };

        self.value.push_str(symbol);
        Ok(self.len())
    }

    fn trim(&mut self) -> Symbol {
        let trimmed = self.value.trim();
        new_string_symbol!(trimmed.to_string())
    }

    pub fn contains(&self, args: Vec<Symbol>) -> Result<Symbol, String> {
        if args.len() != 1 {
            return Err(format!(
                "expected 1 arguments to contains, found {}",
                args.len()
            ));
        }

        let needle = match &args[0] {
            Symbol::String(ss) => &ss.value,
            _ => return Err(format!("string contains expected a string")),
        };
        Ok(Symbol::Boolean(self.value.contains(needle)))
    }

    pub fn call(&mut self, fname: &str, args: Vec<Symbol>) -> Result<Option<Symbol>, String> {
        let option = match fname {
            "insert" => {
                self.insert(args)?;
                None
            }
            "remove" => Some(self.remove(args)?),
            "pop" => self.pop(),
            "len" => Some(self.len()),
            "push" => Some(self.push(args)?),
            "trim" => Some(self.trim()),
            _ => return Err(format!("string has no member '{}'", fname)),
        };

        Ok(option)
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

    pub fn push(&mut self, args: Vec<Symbol>) -> Result<Symbol, String> {
        if args.len() != 1 {
            return Err(format!("incorrect number of arguments to push"));
        }

        let symbol = args.get(0).unwrap().to_owned();
        self.items.push(symbol);
        Ok(self.len())
    }

    pub fn get_mut(&mut self, index: usize) -> Result<&mut Symbol, String> {
        match self.items.get_mut(index) {
            Some(s) => Ok(s),
            None => Err(format!("list index out of range")),
        }
    }

    pub fn get(&self, index: usize) -> Result<&Symbol, String> {
        match self.items.get(index) {
            Some(s) => Ok(s),
            None => Err(format!("list index out of range")),
        }
    }

    pub fn remove(&mut self, args: Vec<Symbol>) -> Result<Symbol, String> {
        if args.len() != 1 {
            return Err(format!("incorrect number of arguments to remove"));
        }

        let index = match args.get(0).unwrap().to_owned() {
            Symbol::Number(index) => index as usize,
            _ => return Err(format!("list indexes must be of type number")),
        };

        if index > self.items.len() {
            return Err(format!("list remove index out of range"));
        }

        Ok(self.items.remove(index))
    }

    pub fn insert(&mut self, args: Vec<Symbol>) -> Result<(), String> {
        if args.len() != 2 {
            return Err(format!(
                "expected 2 arguments to insert, found {}",
                args.len()
            ));
        }

        let index = match args.get(0).unwrap().to_owned() {
            Symbol::Number(index) => index as usize,
            _ => return Err(format!("list indexes must be of type number")),
        };

        if index > self.items.len() {
            return Err(format!("list insert index out of range"));
        }

        let symbol = args.get(1).unwrap().to_owned();
        self.items.insert(index, symbol);
        Ok(())
    }

    pub fn contains(&self, args: Vec<Symbol>) -> Result<Symbol, String> {
        if args.len() != 1 {
            return Err(format!(
                "expected 1 arguments to contains, found {}",
                args.len()
            ));
        }

        let symbol = &args[0];
        Ok(Symbol::Boolean(self.items.contains(symbol)))
    }

    pub fn call(&mut self, fname: &str, args: Vec<Symbol>) -> Result<Option<Symbol>, String> {
        let option = match fname {
            "len" => Some(self.len()),
            "pop" => self.pop(),
            "push" => Some(self.push(args)?),
            "remove" => Some(self.remove(args)?),
            "contains" => Some(self.contains(args)?),
            "insert" => {
                self.insert(args)?;
                None
            }
            _ => return Err(format!("list has no member '{}'", fname)),
        };

        Ok(option)
    }
}

fn compare_literal<T>(left: &T, operator: &TokenType, right: &T) -> Result<bool, String>
where
    T: std::cmp::PartialEq + std::cmp::PartialOrd + std::fmt::Display,
{
    let b = match operator {
        TokenType::GreaterThan => left > right,
        TokenType::LessThan => left < right,
        TokenType::Ge => left >= right,
        TokenType::Le => left <= right,
        _ => {
            return Err(format!(
                "{} {} {}: unable to compare literals",
                left, operator, right
            ))
        }
    };

    Ok(b)
}

fn compare_relational(left: &Symbol, op: &TokenType, right: &Symbol) -> Result<bool, String> {
    match (left, right) {
        (Symbol::Number(lv), Symbol::Number(rv)) => compare_literal(lv, op, rv),
        (Symbol::Boolean(lv), Symbol::Boolean(rv)) => compare_literal(lv, op, rv),
        (Symbol::String(lv), Symbol::String(rv)) => compare_literal(&lv.value, op, &rv.value),
        _ => Err(format!("type mismatch: {} {} {}", left, op, right)),
    }
}

pub fn eval_binary_expression(
    left: &Symbol,
    operator: &TokenType,
    right: &Symbol,
) -> Result<Symbol, String> {
    match operator {
        TokenType::Plus => left + right,
        TokenType::Minus => left - right,
        TokenType::Asterisk => left * right,
        TokenType::ForwardSlash => left / right,
        TokenType::Carat => match (left, right) {
            (Symbol::Number(ln), Symbol::Number(rn)) => Ok(Symbol::Number(ln.powf(*rn))),
            _ => {
                return Err(format!(
                    "can't raise the power of non-number ({}^{})",
                    left, right
                ))
            }
        },
        TokenType::DoubleEquals => Ok(Symbol::Boolean(left == right)),
        TokenType::NotEquals => Ok(Symbol::Boolean(left != right)),
        TokenType::And => Ok(right.clone()),
        TokenType::Or => {
            if left.is_truthy() {
                Ok(left.clone())
            } else {
                Ok(right.clone())
            }
        }
        TokenType::GreaterThan | TokenType::LessThan | TokenType::Ge | TokenType::Le => {
            Ok(Symbol::Boolean(compare_relational(left, operator, right)?))
        }
        _ => return Err(format!("unsupported operator {}", operator)),
    }
}

impl std::ops::Add for &Symbol {
    type Output = Result<Symbol, String>;

    fn add(self, rhs: Self) -> Result<Symbol, String> {
        match (self, rhs) {
            (Symbol::Number(lv), Symbol::Number(rv)) => Ok(Symbol::Number(lv + rv)),
            (Symbol::String(lv), Symbol::String(rv)) => {
                let value = format!("{}{}", lv.value, rv.value);
                Ok(new_string_symbol!(value))
            }
            _ => Err(format!("unsupported operand type for {} + {}", self, rhs)),
        }
    }
}

impl std::ops::Sub for &Symbol {
    type Output = Result<Symbol, String>;

    fn sub(self, rhs: Self) -> Result<Symbol, String> {
        match (self, rhs) {
            (Symbol::Number(lv), Symbol::Number(rv)) => Ok(Symbol::Number(lv - rv)),
            _ => Err(format!("unsupported operand type for {} - {}", self, rhs)),
        }
    }
}

impl std::ops::Mul for &Symbol {
    type Output = Result<Symbol, String>;

    fn mul(self, rhs: Self) -> Result<Symbol, String> {
        match (self, rhs) {
            (Symbol::Number(lv), Symbol::Number(rv)) => Ok(Symbol::Number(lv * rv)),
            _ => Err(format!("unsupported operand type for {} * {}", self, rhs)),
        }
    }
}

impl std::ops::Div for &Symbol {
    type Output = Result<Symbol, String>;

    fn div(self, rhs: Self) -> Result<Symbol, String> {
        match (self, rhs) {
            (Symbol::Number(lv), Symbol::Number(rv)) => Ok(Symbol::Number(lv / rv)),
            _ => Err(format!("unsupported operand type for {} / {}", self, rhs)),
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
    pub fn call(&mut self, call: &str, args: Vec<Symbol>) -> Result<Option<Self>, String> {
        match self {
            Symbol::List(list) => list.call(call, args),
            Symbol::String(ss) => ss.call(call, args),
            _ => Err(format!("{} has no member {}", self.kind(), call)),
        }
    }

    pub fn get_index_mut(&mut self, index: usize) -> Result<&mut Self, String> {
        match self {
            Symbol::List(list) => list.get_mut(index),
            Symbol::String(_) => unimplemented!("mutable index access for strings"),
            _ => Err(format!("object is not indexable")),
        }
    }

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
