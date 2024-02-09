use core::panic;

use crate::{
    ast::ast::{ASTNode, BinaryExpression},
    lexer::{self, TokenType},
};

pub struct Parser {
    lexer: lexer::Lexer,
    curr_token: TokenType,
}

impl Parser {
    pub fn new(src: &str) -> Parser {
        let mut lexer = lexer::Lexer::new(src);
        let curr_token = lexer.next_token();
        Parser { lexer, curr_token }
    }

    pub fn parse(&mut self) -> ASTNode {
        if self.curr_token == TokenType::EOF {
            return ASTNode::Empty;
        }

        self.expression(0)
    }

    fn eat_number(&mut self) -> TokenType {
        match self.curr_token {
            TokenType::Decimal(_) | TokenType::Integer(_) => self.eat(&self.curr_token.clone()),
            _ => panic!("unexpected token {:?}, expected a number", self.curr_token),
        }
    }

    fn eat_operator(&mut self) -> TokenType {
        match self.curr_token {
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Asterisk
            | TokenType::Slash
            | TokenType::Carat => self.eat(&self.curr_token.clone()),
            _ => panic!(
                "unexpected token {:?}, expected an operator",
                self.curr_token
            ),
        }
    }

    fn eat(&mut self, expected_token: &TokenType) -> TokenType {
        if self.curr_token == TokenType::EOF {
            panic!("eof")
        }

        if expected_token != &self.curr_token {
            panic!(
                "unexpected token {:?}, expected {:?}",
                self.curr_token, expected_token
            )
        }

        let previous_token = self.curr_token.clone();
        self.curr_token = self.lexer.next_token();
        return previous_token;
    }

    fn get_precedence(&self, operator: &TokenType) -> usize {
        match operator {
            &TokenType::Carat => 5,
            &TokenType::Asterisk => 3,
            &TokenType::Slash => 3,
            &TokenType::Plus => 2,
            &TokenType::Minus => 2,
            _ => 0,
        }
    }

    /**
     * expression
     *    = prefix (infix)*
     */
    fn expression(&mut self, precedence: usize) -> ASTNode {
        let mut left = self.prefix();

        while self.curr_token != TokenType::EOF
            && precedence < self.get_precedence(&self.curr_token)
        {
            left = self.infix(left, &self.curr_token.clone())
        }

        left
    }

    /**
     * prefix
     *    = parenthesized_expression
     *    / UnaryExpression
     *    / NUMBER
     */
    fn prefix(&mut self) -> ASTNode {
        match &self.curr_token {
            &TokenType::OpenParenthesis => return self.parenthesized_expression(),
            &TokenType::Minus => return self.unary_expression(),
            _ => (),
        };

        let eaten = self.eat_number();
        match eaten {
            TokenType::Decimal(value) => ASTNode::Number(value),
            TokenType::Integer(value) => ASTNode::Number(value as f64),
            _ => panic!("invalid prefix"),
        }
    }

    /**
     * parenthesized_expression
     *    = "(" expression ")"
     */
    fn parenthesized_expression(&mut self) -> ASTNode {
        self.eat(&TokenType::OpenParenthesis);
        let expression = self.expression(0);
        self.eat(&TokenType::CloseParenthesis);
        expression
    }

    /**
     * unary_expression
     *    = "-" expression
     */
    fn unary_expression(&mut self) -> ASTNode {
        self.eat(&TokenType::Minus);
        ASTNode::UnaryExpression(Box::new(self.expression(4)))
    }

    /**
     * infix
     *    = ("+" / "-" / "*" / "/" / "^") expression
     */
    fn infix(&mut self, left: ASTNode, operator: &TokenType) -> ASTNode {
        self.eat_operator();

        let operator_precedence = self.get_precedence(operator);
        let precedence = if operator == &TokenType::Carat {
            operator_precedence - 1
        } else {
            operator_precedence
        };

        return ASTNode::BinaryExpression(BinaryExpression::new(
            left,
            operator.clone(),
            self.expression(precedence),
        ));
    }
}
