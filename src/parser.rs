use core::panic;
use std::collections::HashSet;

use crate::{
    ast::ast::{
        ASTNode, BinaryExpression, BlockStatement, FunctionCall, FunctionExpression, IfStatement,
        MemberExpression, MemberExpressionKind, VariableExpression,
    },
    common::bash,
    lexer::{lexer, token::TokenType},
};

pub struct Parser {
    lexer: lexer::Lexer,
    curr_token: TokenType,
    commands: HashSet<String>,
}

impl Parser {
    pub fn new(src: &str) -> Parser {
        let mut lexer = lexer::Lexer::new(src);
        let curr_token = lexer.next_token();
        Parser {
            lexer,
            curr_token,
            commands: bash::get_commands(),
        }
    }

    fn advance_token(&mut self) {
        self.curr_token = self.lexer.next_token();
    }

    fn advance_cmd_token(&mut self) {
        self.curr_token = self.lexer.next_cmd_token();
    }

    pub fn parse(&mut self) -> ASTNode {
        self.program()
    }

    fn lookahead(&mut self, distance: usize) -> TokenType {
        match distance {
            0 => self.curr_token.clone(),
            _ => self.lexer.lookahead(distance),
        }
    }

    fn eat_literal(&mut self) -> ASTNode {
        let node = match &self.curr_token {
            TokenType::Decimal(dec) => ASTNode::Number(*dec),
            TokenType::Integer(int) => ASTNode::Number(*int as f64),
            TokenType::String(s) => ASTNode::String(s.value.to_string()),
            _ => panic!("unexpected token {}", self.curr_token),
        };

        self.advance_token();
        node
    }

    fn eat_bool(&mut self, b: bool) -> ASTNode {
        self.advance_token();
        ASTNode::Boolean(b)
    }

    fn eat_operator(&mut self) -> TokenType {
        match self.curr_token {
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Asterisk
            | TokenType::ForwardSlash
            | TokenType::Carat
            | TokenType::GreaterThan
            | TokenType::LessThan
            | TokenType::Ge
            | TokenType::Le
            | TokenType::DoubleEquals
            | TokenType::NotEquals
            | TokenType::And
            | TokenType::Or => self.eat(&self.curr_token.clone()),
            _ => panic!("unexpected token {}, expected an operator", self.curr_token),
        }
    }

    fn eat_identifier(&mut self) -> String {
        let curr_token = self.curr_token.clone();
        match &curr_token {
            TokenType::Identifier(ident) | TokenType::EscapedIdentifier(ident) => {
                self.eat(&curr_token);
                ident.clone()
            }
            _ => panic!(
                "unexpected token {}, expected an identifier",
                self.curr_token
            ),
        }
    }

    fn eat(&mut self, expected_token: &TokenType) -> TokenType {
        if self.curr_token == TokenType::EOF {
            panic!("EOF")
        }

        if expected_token != &self.curr_token {
            panic!(
                "unexpected token '{}', expected '{}'",
                self.curr_token, expected_token
            )
        }

        let previous_token = self.curr_token.clone();
        self.advance_token();
        previous_token
    }

    fn get_precedence(&self, operator: &TokenType) -> usize {
        match operator {
            &TokenType::Carat => 5,
            &TokenType::Asterisk => 3,
            &TokenType::ForwardSlash => 3,
            &TokenType::Plus => 2,
            &TokenType::Minus => 2,
            &TokenType::DoubleEquals => 1,
            &TokenType::NotEquals => 1,
            &TokenType::GreaterThan => 1,
            &TokenType::LessThan => 1,
            &TokenType::Ge => 1,
            &TokenType::Le => 1,
            &TokenType::And => 1,
            &TokenType::Or => 1,
            _ => 0,
        }
    }

    /**
     * Program
     *    = statement_list
     */
    fn program(&mut self) -> ASTNode {
        let statement_list = self.statement_list();
        ASTNode::Program(Box::new(statement_list))
    }

    /**
     * statement_list
     *    = statement+
     */
    fn statement_list(&mut self) -> Vec<ASTNode> {
        let mut statements = vec![];

        while self.curr_token != TokenType::EOF {
            if self.curr_token == TokenType::Newline {
                self.eat(&TokenType::Newline);
                continue;
            }

            statements.push(self.statement());

            if self.curr_token != TokenType::EOF {
                self.eat(&TokenType::Newline);
            }
        }

        statements
    }

    /**
     * statement
     *   = variable_statement
     *   / function_expression
     *   / if_statement
     *   / expression
     */
    fn statement(&mut self) -> ASTNode {
        // if self.lookahead(1) == TokenType::Equals {
        //     return self.variable_statement();
        // }

        if let TokenType::Identifier(ident) = &self.curr_token {
            match ident.as_str() {
                "func" => return self.function_expression(),
                "if" => return self.if_statement(),
                _ => (),
            };
        };

        self.expression(0)
    }

    /**
     * if_statement
     *   = "if" block_statement else_statement?
     */
    fn if_statement(&mut self) -> ASTNode {
        self.eat(&TokenType::Identifier("if".to_string()));
        let condition = self.expression(0);
        let consequence = self.block_statement();
        let alternative = match self.else_statement() {
            Some(node) => Some(Box::new(node)),
            None => None,
        };

        ASTNode::IfStatement(IfStatement {
            condition: Box::new(condition),
            consequence: Box::new(consequence),
            alternative,
        })
    }

    /**
     * else_statement
     *   = "else" (if_statement|block_statement)
     */
    fn else_statement(&mut self) -> Option<ASTNode> {
        if self.curr_token != TokenType::Identifier("else".to_string()) {
            return None;
        }
        self.eat_identifier();

        if self.curr_token == TokenType::Identifier("if".to_string()) {
            return Some(self.if_statement());
        }

        Some(self.block_statement())
    }

    /**
     * block_statement
     *   = "{"
     *         block_body
     *     "}"
     */
    fn block_statement(&mut self) -> ASTNode {
        self.eat(&TokenType::OpenBraces);
        self.eat(&TokenType::Newline);
        let body = self.block_body();
        self.eat(&TokenType::CloseBraces);

        ASTNode::BlockStatement(BlockStatement {
            body: Box::new(body),
        })
    }

    /**
     * block_body
     *    = statement+
     */
    fn block_body(&mut self) -> Vec<ASTNode> {
        let mut statements = vec![];
        while self.curr_token != TokenType::CloseBraces {
            if self.curr_token == TokenType::Newline {
                self.eat(&TokenType::Newline);
                continue;
            }

            statements.push(self.statement());
            self.eat(&TokenType::Newline);
        }

        statements
    }

    /**
     * expression
     *  = prefix (infix)*
     */
    fn expression(&mut self, precedence: usize) -> ASTNode {
        let mut left = self.prefix();

        while !self.curr_token.is_end_line() && precedence < self.get_precedence(&self.curr_token) {
            left = self.infix(left, &self.curr_token.clone())
        }

        left
    }

    /**
     * variable_statement
     *   = expression "=" expression
     */
    fn variable_statement(&mut self, lhs: ASTNode) -> ASTNode {
        self.eat(&TokenType::Equals);
        let expression = self.expression(0);

        ASTNode::VariableExpression(VariableExpression {
            lhs: Box::new(lhs),
            rhs: Box::new(expression),
        })
    }

    /**
     * function_expression
     *   = "func" identifier "(" function_expression_args ")" block_statement
     */
    fn function_expression(&mut self) -> ASTNode {
        self.eat(&TokenType::Identifier("func".to_string()));
        let name = self.eat_identifier();
        self.eat(&TokenType::OpenParen);
        let func_args = self.function_expression_args();
        self.eat(&TokenType::CloseParen);
        let body = self.block_statement();

        ASTNode::FunctionExpression(FunctionExpression {
            name,
            body: Box::new(body),
            args: func_args,
        })
    }

    /**
     * function_expression_args
     *   = (identifier,)*
     */
    fn function_expression_args(&mut self) -> Vec<String> {
        if self.curr_token == TokenType::CloseParen {
            return vec![];
        }

        let mut args = vec![];
        loop {
            args.push(self.eat_identifier());
            if self.curr_token == TokenType::CloseParen {
                break;
            }

            self.eat(&TokenType::Comma);
        }
        args
    }

    /**
     * prefix
     *   = parenthesized_expression
     *   / unary_expression
     *   / return_expression
     *   / function_call
     *   / command
     *   / symbol
     */
    fn prefix(&mut self) -> ASTNode {
        match &self.curr_token {
            TokenType::OpenParen => self.parenthesized_expression(),
            TokenType::Minus => self.unary_expression(),
            TokenType::Identifier(ident) => self.parse_identifier(ident.to_owned()),
            TokenType::OpenSqBracket => self.list_literal(),
            _ => self.eat_literal(),
        }
    }

    /**
     * list
     *   = [(expression),*]
     */
    fn list_literal(&mut self) -> ASTNode {
        self.eat(&TokenType::OpenSqBracket);

        let mut items = vec![];
        if self.curr_token == TokenType::CloseSqBracket {
            self.eat(&TokenType::CloseSqBracket);
            return ASTNode::List(Box::new(items));
        }

        loop {
            items.push(self.expression(0));
            if self.curr_token == TokenType::CloseSqBracket {
                self.eat(&TokenType::CloseSqBracket);
                break;
            }
            self.eat(&TokenType::Comma);
        }

        ASTNode::List(Box::new(items))
    }

    fn parse_identifier(&mut self, ident: String) -> ASTNode {
        match self.lookahead(1) {
            TokenType::OpenParen => return ASTNode::FunctionCall(self.function_call()),
            TokenType::OpenSqBracket => return self.member_index_expression(),
            TokenType::Dot => return self.member_call_expression(),
            _ => (),
        };

        match ident.as_str() {
            "return" => self.return_expression(),
            "true" => self.eat_bool(true),
            "false" => self.eat_bool(false),
            s if self.commands.contains(s) => self.command(ident),
            _ => {
                let node = ASTNode::Identifier(self.eat_identifier());
                if self.curr_token == TokenType::Equals {
                    self.variable_statement(node)
                } else {
                    node
                }
            }
        }
    }

    /**
     * identifier "." function_call
     */
    fn member_call_expression(&mut self) -> ASTNode {
        let ident = self.eat_identifier();
        self.eat(&TokenType::Dot);
        let call = self.function_call();

        ASTNode::MemberExpression(MemberExpression {
            identifier: ident,
            kind: Box::new(MemberExpressionKind::Call(call)),
        })
    }

    /**
     * identifier "[" (expression) "]"
     */
    fn member_index_expression(&mut self) -> ASTNode {
        let ident = self.eat_identifier();
        self.eat(&TokenType::OpenSqBracket);
        let expression = self.expression(0);
        self.eat(&TokenType::CloseSqBracket);

        let member_expression = ASTNode::MemberExpression(MemberExpression {
            identifier: ident,
            kind: Box::new(MemberExpressionKind::Index(expression)),
        });

        if self.curr_token == TokenType::Equals {
            self.variable_statement(member_expression)
        } else {
            member_expression
        }
    }

    /*
     * command
     * = command (node)*
     */
    fn command(&mut self, cmd: String) -> ASTNode {
        let mut tokens = vec![ASTNode::String(cmd)];
        self.advance_cmd_token();

        while !self.curr_token.is_end_line() {
            let node = match &self.curr_token {
                TokenType::EscapedIdentifier(ident) => ASTNode::Identifier(ident.to_string()),
                t => ASTNode::String(t.to_string()),
            };

            self.advance_cmd_token();
            tokens.push(node);
        }

        ASTNode::Command(Box::new(tokens))
    }

    /**
     * infix
     *    = ("+" / "-" / "*" / "/" / "^" / "==" / ">" / "<" / ">=" / "<=" / "&&" / "||") expression
     */
    fn infix(&mut self, left: ASTNode, operator: &TokenType) -> ASTNode {
        self.eat_operator();

        let operator_precedence = self.get_precedence(operator);
        let precedence = if operator == &TokenType::Carat {
            operator_precedence - 1
        } else {
            operator_precedence
        };

        ASTNode::BinaryExpression(BinaryExpression {
            left: Box::new(left),
            operator: operator.clone(),
            right: Box::new(self.expression(precedence)),
        })
    }

    /**
     * parenthesized_expression
     *    = "(" expression ")"
     */
    fn parenthesized_expression(&mut self) -> ASTNode {
        self.eat(&TokenType::OpenParen);
        let expression = self.expression(0);
        self.eat(&TokenType::CloseParen);
        expression
    }

    /**
     * return_expression
     *    = "return" expression
     */
    fn return_expression(&mut self) -> ASTNode {
        self.eat(&TokenType::Identifier("return".to_string()));
        let expression = self.expression(0);
        ASTNode::ReturnExpression(Box::new(expression))
    }

    /**
     * function_call
     *    = identifier "(" function_call_args ")"
     */
    fn function_call(&mut self) -> FunctionCall {
        let fname = self.eat_identifier();
        self.eat(&TokenType::OpenParen);
        let args = self.function_call_args();
        self.eat(&TokenType::CloseParen);

        FunctionCall { name: fname, args }
    }

    /**
     * function_call_args
     *   = ((identifier | LITERAL),)*
     */
    fn function_call_args(&mut self) -> Vec<ASTNode> {
        if self.curr_token == TokenType::CloseParen {
            return vec![];
        }

        let mut args = vec![];
        loop {
            let node = match &self.curr_token {
                TokenType::Identifier(_) => ASTNode::Identifier(self.eat_identifier()),
                _ => self.eat_literal(),
            };
            args.push(node);

            if self.curr_token == TokenType::CloseParen {
                break;
            }

            self.eat(&TokenType::Comma);
        }

        args
    }

    /**
     * unary_expression
     *    = "-" expression
     */
    fn unary_expression(&mut self) -> ASTNode {
        self.eat(&TokenType::Minus);
        ASTNode::UnaryExpression(Box::new(self.expression(4)))
    }
}
