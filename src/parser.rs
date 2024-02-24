use core::panic;

use crate::{
    ast::ast::{ASTNode, BinaryExpression, FunctionExpression, VariableExpression},
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
        self.program()
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
            | TokenType::ForwardSlash
            | TokenType::Carat => self.eat(&self.curr_token.clone()),
            _ => panic!(
                "unexpected token {:?}, expected an operator",
                self.curr_token
            ),
        }
    }

    fn eat_identifier(&mut self) -> String {
        let curr_token = self.curr_token.clone();
        match &curr_token {
            TokenType::Identifier(ident) => {
                self.eat(&curr_token);
                return ident.clone();
            }
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
            &TokenType::ForwardSlash => 3,
            &TokenType::Plus => 2,
            &TokenType::Minus => 2,
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

        while self.curr_token != TokenType::EOF && self.curr_token != TokenType::CloseBraces {
            statements.push(self.statement());

            if self.curr_token != TokenType::EOF {
                self.eat(&TokenType::Newline);
            }
        }

        return statements;
    }

    /**
     * statement
     *   = variable_expression
     *   / function_expression
     *   / expression
     */
    fn statement(&mut self) -> ASTNode {
        if self.lexer.lookahead(0) == TokenType::Equals {
            return self.variable_expression();
        }

        if self.curr_token == TokenType::Identifier("func".to_string()) {
            return self.function_expression();
        }

        self.expression(0)
    }

    /**
     * expression
     *  = prefix (infix)*
     */
    fn expression(&mut self, precedence: usize) -> ASTNode {
        let mut left = self.prefix();

        while self.curr_token != TokenType::EOF
            && self.curr_token != TokenType::Newline
            && precedence < self.get_precedence(&self.curr_token)
        {
            left = self.infix(left, &self.curr_token.clone())
        }

        left
    }

    /**
     * variable_expression
     *   = identifier "=" expression
     */
    fn variable_expression(&mut self) -> ASTNode {
        let name = self.eat_identifier();
        self.eat(&TokenType::Equals);
        let expression = self.expression(0);

        ASTNode::VariableExpression(VariableExpression {
            name,
            value: Box::new(expression),
        })
    }

    /**
     * function_expression
     *   = func identifier "(" ")" "{"
     *         expression
     *     "}"
     */
    fn function_expression(&mut self) -> ASTNode {
        self.eat(&TokenType::Identifier("func".to_string()));
        let name = self.eat_identifier();
        self.eat(&TokenType::OpenParenthesis);
        self.eat(&TokenType::CloseParenthesis);
        self.eat(&TokenType::OpenBraces);
        self.eat(&TokenType::Newline);
        let statement_list = self.statement_list();
        self.eat(&TokenType::CloseBraces);

        ASTNode::FunctionExpression(FunctionExpression {
            name,
            body: Box::new(statement_list),
        })
    }

    /**
     * prefix
     *    = parenthesized_expression
     *    / unary_expression
     *    / NUMBER
     */
    fn prefix(&mut self) -> ASTNode {
        match &self.curr_token {
            &TokenType::OpenParenthesis => return self.parenthesized_expression(),
            &TokenType::Minus => return self.unary_expression(),
            &TokenType::Identifier(_) => return self.variable_statement(),
            _ => (),
        };

        match self.eat_number() {
            TokenType::Decimal(value) => ASTNode::Number(value),
            TokenType::Integer(value) => ASTNode::Number(value as f64),
            _ => panic!("invalid prefix"),
        }
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

        ASTNode::BinaryExpression(BinaryExpression {
            left: Box::new(left),
            operator: operator.clone(),
            right: Box::new(self.expression(precedence)),
        })
    }

    /**
     * Variable
     *    = IDENTIFIER
     */
    fn variable_statement(&mut self) -> ASTNode {
        let name = self.eat_identifier();
        ASTNode::Variable(name)
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
}
