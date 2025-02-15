use crate::{
    LoxError, Result,
    expr::{Assign, Binary, Call, Expr, Grouping, Literal, Logical, Unary, Variable},
    stmt::{Block, Expression, If, Print, Stmt, Var, While},
    token::Token,
    token_type::TokenType,
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut statements: Vec<Stmt> = Vec::new();
        let mut had_error = false;
        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    had_error = true;
                    self.synchronize();
                    eprintln!("Parsing error {e}");
                }
            }
        }
        if had_error { Err(LoxError::Fatal) } else { Ok(statements) }
    }
}

// Declarations
impl Parser {
    fn declaration(&mut self) -> Result<Stmt> {
        if self.match_advance(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;
        let initializer = if self.match_advance(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration")?;

        Ok(Var::stmt(name, initializer))
    }
}

// Statements
impl Parser {
    fn statement(&mut self) -> Result<Stmt> {
        if self.match_advance(&[TokenType::Print]) {
            return self.print_statement();
        }
        if self.match_advance(&[TokenType::While]) {
            return self.while_statement();
        }
        if self.match_advance(&[TokenType::LeftBrace]) {
            return Ok(Block::stmt(self.block()?));
        }
        if self.match_advance(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.match_advance(&[TokenType::For]) {
            return self.for_statement();
        }
        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value")?;
        Ok(Print::stmt(value))
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;

        Ok(While::stmt(condition, body))
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition")?;

        let then_branch = self.statement()?;
        let else_branch = if self.match_advance(&[TokenType::Else]) {
            Some(self.statement()?)
        } else {
            None
        };

        Ok(If::stmt(condition, then_branch, else_branch))
    }

    /// De-sugar a for statement into a while statement
    fn for_statement(&mut self) -> Result<Stmt> {
        /* for (var i = 0; i < 10; i = i + 1) {
         *    print i;
         *  }
         */
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;
        // `var i = 0;`, could also be empty, or just an expression which
        // we'd treat as a statement to keep things tidy
        let initializer = match self.peek().typ {
            TokenType::Semicolon => {
                self.advance();
                None
            }
            TokenType::Var => {
                self.advance();
                Some(self.var_declaration()?)
            }
            _ => Some(self.expression_statement()?),
        };

        // `i < 10;`, if not present use `true` instead
        let condition = match self.check(&TokenType::Semicolon) {
            true => Literal::expr(true.into()),
            false => self.expression()?,
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition")?;

        // `i = i + 1;`, could also be empty
        let increment = match self.check(&TokenType::RightParen) {
            true => None,
            false => Some(self.expression()?),
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        // `{ print i; }`
        let mut body = self.statement()?;

        // Now, build out the while statement, working backwards
        if let Some(incr) = increment {
            /* {
             *   { print i; }
             *   i = i + 1;
             * }
             */
            body = Block::stmt(vec![body, Expression::stmt(incr)]);
        }
        /* while (i < 10) {
         *   { print i; }
         *   i = i + 1;
         * }
         */
        body = While::stmt(condition, body);

        /* {
         *   // scope `var` to just this block
         *   var i = 0;
         *   while (i < 10) {
         *     { print i; }
         *     i = i + 1;
         *   }
         * }
         */
        if let Some(init) = initializer {
            body = Block::stmt(vec![init, body]);
        }

        // boom!
        Ok(body)
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression")?;
        Ok(Expression::stmt(expr))
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;

        Ok(statements)
    }
}

// Expressions
impl Parser {
    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;

        if self.match_advance(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(var) = expr {
                let name = var.name;
                return Ok(Assign::expr(name, value));
            }

            Err(error(&equals, "Invalid assignment target."))
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;

        while self.match_advance(&[TokenType::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Logical::expr(expr, operator, right);
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while self.match_advance(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Logical::expr(expr, operator, right);
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while self.match_advance(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Binary::expr(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;
        while self.match_advance(&[TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Binary::expr(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;
        while self.match_advance(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Binary::expr(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;
        while self.match_advance(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Binary::expr(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.match_advance(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Unary::expr(operator, right));
        }
        self.call()
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_advance(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut arguments = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(error(self.peek(), "Can't have more than 255 arguments."));
                }
                arguments.push(self.expression()?);
                if !self.match_advance(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Call::expr(callee, paren, arguments))
    }

    fn primary(&mut self) -> Result<Expr> {
        match self.advance().typ {
            TokenType::False => Ok(Literal::expr(false.into())),
            TokenType::True => Ok(Literal::expr(true.into())),
            TokenType::Nil => Ok(Literal::expr(().into())),
            TokenType::Number | TokenType::String => Ok(Literal::expr(self.previous().literal)),
            TokenType::Identifier => Ok(Variable::expr(self.previous())),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
                Ok(Grouping::expr(expr))
            }
            _ => Err(error(&self.previous(), "Expected an expression")),
        }
    }
}

// Helpers
impl Parser {
    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&mut self, typ: TokenType, msg: &str) -> Result<Token> {
        if self.check(&typ) {
            return Ok(self.advance());
        }

        Err(error(self.peek(), msg))
    }

    /// If any of the token types are the next token, advance and return true
    /// Otherwise, return false and do not advance
    fn match_advance(&mut self, typs: &[TokenType]) -> bool {
        if typs.iter().any(|t| self.check(t)) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, typ: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        &self.peek().typ == typ
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Returns the next token and advances over it (if not at the end)
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().typ == TokenType::Eof
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().typ == TokenType::Semicolon {
                return;
            }
            match self.peek().typ {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
}

fn error(token: &Token, message: &str) -> LoxError {
    let message = message.to_string();
    match token.typ {
        TokenType::Eof => LoxError::Parsing {
            line: token.line,
            whence: "at end".to_string(),
            message,
        },
        _ => LoxError::Parsing {
            line: token.line,
            whence: format!("at '{}'", token.lexeme),
            message,
        },
    }
}
