use crate::lexer::token::TokenType;

#[derive(Debug, Clone)]
pub enum ASTNode {
    Program(Box<Vec<ASTNode>>),

    IfStatement(IfStatement),
    BlockStatement(BlockStatement),
    ReturnStatement(Box<ASTNode>),
    ForStatement(ForStatement),

    MemberExpression(MemberExpression),
    IndexExpression(IndexExpression),
    FunctionStatement(FunctionStatement),
    CallExpression(CallExpression),

    VariableExpression(VariableExpression),
    BinaryExpression(BinaryExpression),
    UnaryExpression(Box<ASTNode>),
    RangeExpression(RangeExpression),

    Number(f64),
    Boolean(bool),
    String(String),
    Identifier(String),
    None,
    List(Box<Vec<ASTNode>>),

    Command(Box<Vec<ASTNode>>),
}

#[derive(Debug, Clone)]
pub struct ForStatement {
    pub variable: String,
    pub iterable: Box<Iterable>,
    pub body: Box<ASTNode>,
}

#[derive(Debug, Clone)]
pub enum Iterable {
    RangeExpression(RangeExpression),
    Collection(ASTNode),
}

#[derive(Debug, Clone)]
pub struct RangeExpression {
    pub start: Box<ASTNode>,
    pub end: Box<ASTNode>,
    pub increment: Option<Box<ASTNode>>,
}

// x.y = MemberExpression (base = idenitifier(x), identifier(argv))
// x[0] = MemberExpression (index)
// process.argv[0] = MemberExpression (base = MemberExpression(base = process, identifier(argv)), Index(0))
// x.foo() CallExpression (callee/base = MemberExpression)
// foo() CallExpression (callee/base = identifier)

#[derive(Debug, Clone)]
pub struct IndexExpression {
    pub base: Box<ASTNode>,
    pub index: Box<ASTNode>,
}

#[derive(Debug, Clone)]
pub struct MemberExpression {
    //  pub identifier: String, // this needs to be abstract
    pub base: Box<ASTNode>,
    pub property: String,
}

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Box<ASTNode>,
    pub operator: TokenType,
    pub right: Box<ASTNode>,
}

#[derive(Debug, Clone)]
pub struct VariableExpression {
    pub lhs: Box<ASTNode>,
    pub rhs: Box<ASTNode>,
}

#[derive(Debug, Clone)]
pub struct FunctionStatement {
    pub name: String,
    pub body: Box<ASTNode>,
    pub args: Vec<String>,
}

impl PartialEq for FunctionStatement {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct CallExpression {
    pub base: Box<ASTNode>,
    pub args: Vec<ASTNode>,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Box<ASTNode>,
    pub consequence: Box<ASTNode>,
    pub alternative: Option<Box<ASTNode>>,
}

#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub body: Box<Vec<ASTNode>>,
}
