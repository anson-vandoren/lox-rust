use crate::{
    expr::{Binary, Expr, Grouping, Literal, Unary},
    token::Token,
    token_type::TokenType,
    LoxError,
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

type Result<T> = std::result::Result<T, LoxError>;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Option<Expr>> {
        if self.peek().typ == TokenType::Eof {
            return Ok(None);
        }
        Some(self.expression()).transpose()
    }

    fn expression(&mut self) -> Result<Expr> {
        self.equality()
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

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;
        while self.match_advance(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
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
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        match self.advance().typ {
            TokenType::False => Ok(Literal::expr(false.into())),
            TokenType::True => Ok(Literal::expr(true.into())),
            TokenType::Nil => Ok(Literal::expr(().into())),
            TokenType::Number | TokenType::String => Ok(Literal::expr(self.previous().literal)),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
                Ok(Grouping::expr(expr))
            }
            _ => Err(error(&self.previous(), "Expected an expression")),
        }
    }

    fn consume(&mut self, typ: TokenType, msg: &str) -> Result<Token> {
        if self.check(&typ) {
            return Ok(self.advance());
        }

        Err(error(self.peek(), msg))
    }

    fn match_advance(&mut self, typs: &[TokenType]) -> bool {
        for typ in typs {
            if self.check(typ) {
                self.advance();
                return true;
            }
        }

        false
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
