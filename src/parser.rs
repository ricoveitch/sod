use std::collections::HashSet;

use crate::{
    ast::ast::{
        self, ASTNode, BinaryExpression, BlockStatement, CallExpression, ForStatement,
        FunctionStatement, IfStatement, IndexExpression, MemberExpression, RangeExpression,
        TemplateString, TemplateToken, VariableExpression,
    },
    commands,
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
            commands: commands::get_commands(),
        }
    }

    fn advance_token(&mut self) {
        self.curr_token = self.lexer.next_token();
    }

    fn advance_cmd_token(&mut self) {
        self.curr_token = self.lexer.next_cmd_token();
    }

    pub fn parse(&mut self) -> Result<ASTNode, String> {
        self.program()
    }

    fn lookahead(&mut self, distance: usize) -> TokenType {
        match distance {
            0 => self.curr_token.clone(),
            _ => self.lexer.lookahead(distance),
        }
    }

    fn eat_literal(&mut self) -> Result<ASTNode, String> {
        let node = match &self.curr_token {
            TokenType::Decimal(dec) => ASTNode::Number(*dec),
            TokenType::Integer(int) => {
                let number = ASTNode::Number(*int as f64);
                match self.lookahead(1) {
                    TokenType::Dot => {
                        self.advance_token();
                        return Ok(ASTNode::RangeExpression(self.range_expression(number)?));
                    }
                    _ => number,
                }
            }
            TokenType::String(s) => ASTNode::String(s.to_string()),
            TokenType::TemplateString(ts) => self.read_template_string(ts.as_str()),
            _ => return Err(format!("unexpected token '{}'", self.curr_token)),
        };

        self.advance_token();
        Ok(node)
    }

    fn eat_bool(&mut self, b: bool) -> ASTNode {
        self.advance_token();
        ASTNode::Boolean(b)
    }

    fn eat_operator(&mut self) -> Result<TokenType, String> {
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
            _ => Err(format!(
                "unexpected token '{}', expected an operator",
                self.curr_token
            )),
        }
    }

    fn eat_identifier(&mut self) -> Result<String, String> {
        let curr_token = self.curr_token.clone();
        match &curr_token {
            TokenType::Identifier(ident) | TokenType::EscapedIdentifier(ident) => {
                self.eat(&curr_token)?;
                Ok(ident.clone())
            }
            _ => Err(format!(
                "unexpected token '{}', expected an identifier",
                self.curr_token
            )),
        }
    }

    fn eat(&mut self, expected_token: &TokenType) -> Result<TokenType, String> {
        if self.curr_token == TokenType::EOF {
            return Err(format!("EOF"));
        }

        if expected_token != &self.curr_token {
            return Err(format!("unexpected token '{}'", self.curr_token));
        }

        let previous_token = self.curr_token.clone();
        self.advance_token();
        Ok(previous_token)
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
    fn program(&mut self) -> Result<ASTNode, String> {
        let statement_list = self.statement_list()?;
        Ok(ASTNode::Program(Box::new(statement_list)))
    }

    /**
     * statement_list
     *    = statement+
     */
    fn statement_list(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut statements = vec![];

        while self.curr_token != TokenType::EOF {
            if self.curr_token == TokenType::Newline {
                self.eat(&TokenType::Newline)?;
                continue;
            }

            statements.push(self.statement()?);

            if self.curr_token != TokenType::EOF {
                self.eat(&TokenType::Newline)?;
            }
        }

        Ok(statements)
    }

    /**
     * statement
     *   = variable_statement
     *   / function_expression
     *   / if_statement
     *   / expression
     */
    fn statement(&mut self) -> Result<ASTNode, String> {
        if let TokenType::Identifier(ident) = &self.curr_token {
            match ident.as_str() {
                "func" => return Ok(self.function_expression()?),
                "if" => return self.if_statement(),
                "for" => return self.for_statement(),
                _ => (),
            };
        };

        self.expression(0)
    }

    /**
     * for_statement
     *   = "for" identifier range_expression block_statement
     */
    fn for_statement(&mut self) -> Result<ASTNode, String> {
        self.eat(&TokenType::Identifier("for".to_string()))?;
        let variable = self.eat_identifier()?;
        self.eat(&TokenType::Identifier("in".to_string()))?;
        let iterable = self.iterable()?;
        let body = self.block_statement()?;

        Ok(ASTNode::ForStatement(ForStatement {
            variable,
            iterable: Box::new(iterable),
            body: Box::new(body),
        }))
    }

    /**
     * iterable
     *   = (range_expression | expression)
     */
    fn iterable(&mut self) -> Result<ast::Iterable, String> {
        let expression = self.expression(0)?;
        let iterable = match self.curr_token {
            TokenType::Dot => ast::Iterable::RangeExpression(self.range_expression(expression)?),
            _ => ast::Iterable::Collection(expression),
        };

        Ok(iterable)
    }

    /**
     *  range_expression
     *   = start_expression ".." end_expression (".." increment_expression)?
     */
    fn range_expression(&mut self, start: ASTNode) -> Result<RangeExpression, String> {
        self.eat(&TokenType::Dot)?;
        self.eat(&TokenType::Dot)?;
        let (end, increment) = match self.expression(0)? {
            ASTNode::RangeExpression(re) => (*re.start, Some(Box::new(*re.end))),
            node => (node, None),
        };

        Ok(RangeExpression {
            start: Box::new(start),
            end: Box::new(end),
            increment,
        })
    }

    /**
     * if_statement
     *   = "if" block_statement else_statement?
     */
    fn if_statement(&mut self) -> Result<ASTNode, String> {
        self.eat(&TokenType::Identifier("if".to_string()))?;
        let condition = self.expression(0)?;
        let consequence = self.block_statement()?;
        let alternative = match self.else_statement()? {
            Some(node) => Some(Box::new(node)),
            None => None,
        };

        Ok(ASTNode::IfStatement(IfStatement {
            condition: Box::new(condition),
            consequence: Box::new(consequence),
            alternative,
        }))
    }

    /**
     * else_statement
     *   = "else" (if_statement|block_statement)
     */
    fn else_statement(&mut self) -> Result<Option<ASTNode>, String> {
        if self.curr_token != TokenType::Identifier("else".to_string()) {
            return Ok(None);
        }
        self.eat_identifier()?;

        if self.curr_token == TokenType::Identifier("if".to_string()) {
            return Ok(Some(self.if_statement()?));
        }

        Ok(Some(self.block_statement()?))
    }

    /**
     * block_statement
     *   = "{"
     *         block_body
     *     "}"
     */
    fn block_statement(&mut self) -> Result<ASTNode, String> {
        self.eat(&TokenType::OpenBraces)?;
        self.eat(&TokenType::Newline)?;
        let body = self.block_body()?;
        self.eat(&TokenType::CloseBraces)?;

        Ok(ASTNode::BlockStatement(BlockStatement {
            body: Box::new(body),
        }))
    }

    /**
     * block_body
     *    = statement+
     */
    fn block_body(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut statements = vec![];
        while self.curr_token != TokenType::CloseBraces {
            if self.curr_token == TokenType::Newline {
                self.eat(&TokenType::Newline)?;
                continue;
            }

            statements.push(self.statement()?);
            self.eat(&TokenType::Newline)?;
        }

        Ok(statements)
    }

    /**
     * expression
     *  = prefix (infix)*
     */
    fn expression(&mut self, precedence: usize) -> Result<ASTNode, String> {
        let mut left = self.prefix()?;

        while !self.curr_token.is_end_line() && precedence < self.get_precedence(&self.curr_token) {
            left = self.infix(left, &self.curr_token.clone())?;
        }

        Ok(left)
    }

    /**
     * variable_statement
     *   = expression "=" expression
     */
    fn variable_statement(&mut self, lhs: ASTNode) -> Result<ASTNode, String> {
        self.eat(&TokenType::Equals)?;
        let expression = self.expression(0)?;

        Ok(ASTNode::VariableExpression(VariableExpression {
            lhs: Box::new(lhs),
            rhs: Box::new(expression),
        }))
    }

    /**
     * function_expression
     *   = "func" identifier "(" function_expression_args ")" block_statement
     */
    fn function_expression(&mut self) -> Result<ASTNode, String> {
        self.eat(&TokenType::Identifier("func".to_string()))?;
        let name = self.eat_identifier()?;
        self.eat(&TokenType::OpenParen)?;
        let func_args = self.function_expression_args()?;
        self.eat(&TokenType::CloseParen)?;
        let body = self.block_statement()?;

        Ok(ASTNode::FunctionStatement(FunctionStatement {
            name,
            body: Box::new(body),
            args: func_args,
        }))
    }

    /**
     * function_expression_args
     *   = (identifier,)*
     */
    fn function_expression_args(&mut self) -> Result<Vec<String>, String> {
        if self.curr_token == TokenType::CloseParen {
            return Ok(vec![]);
        }

        let mut args = vec![];
        loop {
            args.push(self.eat_identifier()?);
            if self.curr_token == TokenType::CloseParen {
                break;
            }

            self.eat(&TokenType::Comma)?;
        }
        Ok(args)
    }

    /**
     * prefix
     *   = parenthesized_expression
     *   / unary_expression
     *   / return_expression
     *   / call_expression
     *   / command
     *   / symbol
     */
    fn prefix(&mut self) -> Result<ASTNode, String> {
        match &self.curr_token {
            TokenType::OpenParen => self.parenthesized_expression(),
            TokenType::Minus => self.unary_expression(),
            TokenType::Identifier(ident) => self.parse_identifier(ident.to_owned()),
            TokenType::OpenSqBracket => return self.list_literal(),
            _ => return self.eat_literal(),
        }
    }

    /**
     * list
     *   = [(expression),*]
     */
    fn list_literal(&mut self) -> Result<ASTNode, String> {
        self.eat(&TokenType::OpenSqBracket)?;

        let mut items = vec![];
        if self.curr_token == TokenType::CloseSqBracket {
            self.eat(&TokenType::CloseSqBracket)?;
            return Ok(ASTNode::List(Box::new(items)));
        }

        loop {
            items.push(self.expression(0)?);
            if self.curr_token == TokenType::CloseSqBracket {
                self.eat(&TokenType::CloseSqBracket)?;
                break;
            }
            self.eat(&TokenType::Comma)?;
        }

        Ok(ASTNode::List(Box::new(items)))
    }

    fn parse_identifier(&mut self, ident: String) -> Result<ASTNode, String> {
        match self.lookahead(1) {
            TokenType::OpenParen => {
                self.advance_token();
                return self.call_expression(ASTNode::Identifier(ident));
            }
            TokenType::OpenSqBracket | TokenType::Dot => {
                self.advance_token();
                return self.member_expression(ASTNode::Identifier(ident));
            }
            _ => (),
        };

        let node = match ident.as_str() {
            "return" => self.return_expression()?,
            "true" => self.eat_bool(true),
            "false" => self.eat_bool(false),
            "none" => {
                self.eat(&TokenType::Identifier(ident))?;
                ASTNode::None
            }
            s if self.commands.contains(s) => self.command(ident),
            _ => {
                let node = ASTNode::Identifier(self.eat_identifier()?);
                if self.curr_token == TokenType::Equals {
                    self.variable_statement(node)?
                } else {
                    node
                }
            }
        };

        Ok(node)
    }

    /**
     * identifier member_prefix_expression
     */
    fn member_expression(&mut self, base: ASTNode) -> Result<ASTNode, String> {
        let mut base = base;
        loop {
            let (new_base, more) = self.member_prefix_expression(base)?;
            base = new_base;
            if !more {
                break;
            }
        }

        // this could be parsed better
        if self.curr_token == TokenType::Equals {
            self.variable_statement(base)
        } else {
            Ok(base)
        }
    }

    /**
     * member_prefix_expression =
     *    member_expression | index_expression | call_expression
     */
    fn member_prefix_expression(&mut self, base: ASTNode) -> Result<(ASTNode, bool), String> {
        let expression = match &self.curr_token {
            &TokenType::Dot => {
                self.eat(&TokenType::Dot)?;
                let property = self.eat_identifier()?;
                let me = MemberExpression {
                    base: Box::new(base),
                    property,
                };

                ASTNode::MemberExpression(me)
            }
            &TokenType::OpenSqBracket => {
                self.eat(&TokenType::OpenSqBracket)?;
                let index = self.expression(0)?;
                self.eat(&TokenType::CloseSqBracket)?;

                ASTNode::IndexExpression(IndexExpression {
                    base: Box::new(base),
                    index: Box::new(index),
                })
            }
            &TokenType::OpenParen => self.call_expression(base)?,
            _ => return Ok((base, false)),
        };

        Ok((expression, true))
    }

    /*
     * command
     * = command (node)*
     */
    fn command(&mut self, cmd: String) -> ASTNode {
        let mut tokens = vec![ASTNode::String(cmd)];

        let mut prev = self.curr_token.clone();
        self.advance_cmd_token();

        loop {
            if self.curr_token.is_end_line() && prev != TokenType::BackSlash {
                break;
            }

            let node = match &self.curr_token {
                TokenType::EscapedIdentifier(ident) => ASTNode::Identifier(ident.to_string()),
                TokenType::TemplateString(s) => self.read_template_string(s.as_str()),
                t => ASTNode::String(t.to_string()),
            };

            prev = self.curr_token.clone();
            self.advance_cmd_token();
            tokens.push(node);
        }

        ASTNode::Command(Box::new(tokens))
    }

    /**
     * infix
     *    = ("+" / "-" / "*" / "/" / "^" / "==" / ">" / "<" / ">=" / "<=" / "&&" / "||") expression
     */
    fn infix(&mut self, left: ASTNode, operator: &TokenType) -> Result<ASTNode, String> {
        self.eat_operator()?;

        let operator_precedence = self.get_precedence(operator);
        let precedence = if operator == &TokenType::Carat {
            operator_precedence - 1
        } else {
            operator_precedence
        };

        Ok(ASTNode::BinaryExpression(BinaryExpression {
            left: Box::new(left),
            operator: operator.clone(),
            right: Box::new(self.expression(precedence)?),
        }))
    }

    /**
     * parenthesized_expression
     *    = "(" expression ")"
     */
    fn parenthesized_expression(&mut self) -> Result<ASTNode, String> {
        self.eat(&TokenType::OpenParen)?;
        let expression = self.expression(0)?;
        self.eat(&TokenType::CloseParen)?;
        Ok(expression)
    }

    /**
     * return_expression
     *    = "return" expression
     */
    fn return_expression(&mut self) -> Result<ASTNode, String> {
        self.eat(&TokenType::Identifier("return".to_string()))?;
        let expression = self.expression(0)?;
        Ok(ASTNode::ReturnStatement(Box::new(expression)))
    }

    /**
     * call_expression
     *    = identifier "(" call_expression_args ")"
     */
    fn call_expression(&mut self, base: ASTNode) -> Result<ASTNode, String> {
        self.eat(&TokenType::OpenParen)?;
        let args = self.call_expression_args()?;
        self.eat(&TokenType::CloseParen)?;

        let call_expression = ASTNode::CallExpression(CallExpression {
            base: Box::new(base),
            args,
        });

        if self.curr_token == TokenType::Dot {
            return self.member_expression(call_expression);
        }

        Ok(call_expression)
    }

    /**
     * call_expression_args
     *   = "(" (expression,)* ")"
     */
    fn call_expression_args(&mut self) -> Result<Vec<ASTNode>, String> {
        if self.curr_token == TokenType::CloseParen {
            return Ok(vec![]);
        }

        let mut args = vec![];
        loop {
            args.push(self.expression(0)?);

            if self.curr_token == TokenType::CloseParen {
                break;
            }

            self.eat(&TokenType::Comma)?;
        }

        Ok(args)
    }

    /**
     * unary_expression
     *    = "-" expression
     */
    fn unary_expression(&mut self) -> Result<ASTNode, String> {
        self.eat(&TokenType::Minus)?;
        Ok(ASTNode::UnaryExpression(Box::new(self.expression(4)?)))
    }

    fn read_template_string(&self, value: &str) -> ASTNode {
        let mut tokens = vec![];

        let mut tail = 0;
        while tail < value.len() {
            if value.chars().nth(tail).unwrap() == '$' {
                tail += 1;
                let head = tail;
                while tail < value.len() && value.chars().nth(tail).unwrap() != ' ' {
                    tail += 1;
                }
                if tail == head {
                    tokens.push(TemplateToken::Literal("$".to_string()))
                } else {
                    tokens.push(TemplateToken::Expression(value[head..tail].to_string()))
                }
            } else {
                let head = tail;
                while tail < value.len() && value.chars().nth(tail).unwrap() != '$' {
                    tail += 1;
                }
                tokens.push(TemplateToken::Literal(value[head..tail].to_string()))
            }
        }

        ASTNode::TemplateString(TemplateString { tokens })
    }
}
