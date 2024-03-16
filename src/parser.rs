use core::panic;

use crate::{
    ast::{
        ast::{
            ASTNode, BinaryExpression, FunctionCall, FunctionExpression, IfStatement,
            VariableExpression,
        },
        symbol::Symbol,
    },
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

    fn lookahead(&mut self, distance: usize) -> TokenType {
        match distance {
            0 => self.curr_token.clone(),
            _ => self.lexer.lookahead(distance),
        }
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
            | TokenType::Carat
            | TokenType::GreaterThan
            | TokenType::LessThan
            | TokenType::GreaterThanOrEqualTo
            | TokenType::LessThanOrEqualTo
            | TokenType::And
            | TokenType::Or => self.eat(&self.curr_token.clone()),
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
                ident.clone()
            }
            _ => panic!(
                "unexpected token {:?}, expected an identifier",
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
            &TokenType::GreaterThan => 1,
            &TokenType::LessThan => 1,
            &TokenType::GreaterThanOrEqualTo => 1,
            &TokenType::LessThanOrEqualTo => 1,
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

        while self.curr_token != TokenType::EOF && self.curr_token != TokenType::CloseBraces {
            statements.push(self.statement());

            if self.curr_token != TokenType::EOF {
                self.eat(&TokenType::Newline);
            }
        }

        statements
    }

    /**
     * statement
     *   = variable_expression
     *   / function_expression
     *   / if_statement
     *   / expression
     */
    fn statement(&mut self) -> ASTNode {
        if self.lookahead(1) == TokenType::Equals {
            return self.variable_expression();
        }

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
     *   = "if" expression "{"
     *         statement_list
     *     "}"
     */
    fn if_statement(&mut self) -> ASTNode {
        self.eat(&TokenType::Identifier("if".to_string()));
        let condition = self.expression(0);
        self.eat(&TokenType::OpenBraces);
        self.eat(&TokenType::Newline);
        let consequence = self.statement_list();
        self.eat(&TokenType::CloseBraces);

        ASTNode::IfStatement(IfStatement {
            condition: Box::new(condition),
            consequence: Box::new(consequence),
            alternative: None,
        })
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
     *   = "func" identifier "(" function_expression_args ")" "{"
     *         statement_list
     *     "}"
     */
    fn function_expression(&mut self) -> ASTNode {
        self.eat(&TokenType::Identifier("func".to_string()));
        let name = self.eat_identifier();
        self.eat(&TokenType::OpenParenthesis);
        let func_args = self.function_expression_args();
        self.eat(&TokenType::CloseParenthesis);
        self.eat(&TokenType::OpenBraces);
        self.eat(&TokenType::Newline);
        let statement_list = self.statement_list();
        self.eat(&TokenType::CloseBraces);

        ASTNode::FunctionExpression(FunctionExpression {
            name,
            body: Box::new(statement_list),
            args: func_args,
        })
    }

    /**
     * function_expression_args
     *   = (identifier,)*
     */
    fn function_expression_args(&mut self) -> Vec<String> {
        if self.curr_token == TokenType::CloseParenthesis {
            return vec![];
        }

        let mut args = vec![];
        loop {
            args.push(self.eat_identifier());
            if self.curr_token == TokenType::CloseParenthesis {
                break;
            }

            self.eat(&TokenType::Comma);
        }
        args
    }

    /**
     * prefix
     *    = parenthesized_expression
     *    / unary_expression
     *    / return_expression
     *    / NUMBER
     */
    fn prefix(&mut self) -> ASTNode {
        match &self.curr_token {
            TokenType::OpenParenthesis => return self.parenthesized_expression(),
            TokenType::Minus => return self.unary_expression(),
            TokenType::Identifier(_) => return self.identifier(),
            _ => (),
        };

        match self.eat_number() {
            TokenType::Decimal(value) => ASTNode::Number(value),
            TokenType::Integer(value) => ASTNode::Number(value as f64),
            _ => panic!("invalid prefix"),
        }
    }

    fn identifier(&mut self) -> ASTNode {
        let ident = self.eat_identifier();
        if self.curr_token == TokenType::OpenParenthesis {
            return self.function_call(ident);
        }

        match ident.as_str() {
            "return" => self.return_expression(),
            "true" => ASTNode::Boolean(true),
            "false" => ASTNode::Boolean(false),
            _ => ASTNode::Variable(ident),
        }
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
        self.eat(&TokenType::OpenParenthesis);
        let expression = self.expression(0);
        self.eat(&TokenType::CloseParenthesis);
        expression
    }

    /**
     * return_expression
     *    = "return" expression
     */
    fn return_expression(&mut self) -> ASTNode {
        let expression = self.expression(0);
        ASTNode::ReturnExpression(Box::new(expression))
    }

    /**
     * function_call
     *    = identifier "(" function_call_args ")"
     */
    fn function_call(&mut self, fname: String) -> ASTNode {
        self.eat(&TokenType::OpenParenthesis);
        let args = self.function_call_args();
        self.eat(&TokenType::CloseParenthesis);
        ASTNode::FunctionCall(FunctionCall { name: fname, args })
    }

    /**
     * function_call_args
     *   = ((identifier | NUMBER),)*
     */
    fn function_call_args(&mut self) -> Vec<Symbol> {
        if self.curr_token == TokenType::CloseParenthesis {
            return vec![];
        }

        let mut args = vec![];
        loop {
            match &self.curr_token {
                TokenType::Identifier(_) => args.push(Symbol::Variable(self.eat_identifier())),
                _ => match self.eat_number() {
                    TokenType::Decimal(value) => args.push(Symbol::Number(value)),
                    TokenType::Integer(value) => args.push(Symbol::Number(value as f64)),
                    _ => panic!("invalid function argument"),
                },
            };
            if self.curr_token == TokenType::CloseParenthesis {
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
