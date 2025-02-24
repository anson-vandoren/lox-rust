use ordered_float::OrderedFloat;
use tracing::{error, instrument};

use crate::{LoxError, Result, object::Literal, token::Token, token_type::TokenType};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    current: usize,
    start: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            current: 0,
            start: 0,
            line: 1,
        }
    }

    #[instrument(skip(self), err, level = "trace")]
    pub fn scan_tokens(mut self) -> Result<Vec<Token>> {
        let mut had_error = false;
        let eof = self.source.len();

        while self.current < eof {
            self.start = self.current;
            if self
                .scan_token()
                .map_err(|error| error!(?error, "Error while scanning"))
                .is_err()
            {
                had_error = true;
            }
        }
        self.tokens
            .push(Token::new(TokenType::Eof, "", Literal::Null, self.line));

        match had_error {
            false => Ok(self.tokens),
            true => Err(LoxError::Fatal {}),
        }
    }

    #[instrument(skip(self), err, level = "trace")]
    fn scan_token(&mut self) -> Result<()> {
        let c = self.advance();
        let mut if_equals_else = |is_equal: TokenType, not_equal: TokenType| {
            let token_type = if self.advance_if_is('=') {
                is_equal
            } else {
                not_equal
            };
            self.add_token(token_type);
        };
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => if_equals_else(TokenType::BangEqual, TokenType::Bang),
            '=' => if_equals_else(TokenType::EqualEqual, TokenType::Equal),
            '<' => if_equals_else(TokenType::LessEqual, TokenType::Less),
            '>' => if_equals_else(TokenType::GreaterEqual, TokenType::Greater),
            '/' => {
                match self.peek() {
                    '/' => {
                        // It's a single-line comment
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    }
                    '*' => {
                        // It's a multi-line comment
                        self.advance();
                        while self.peek() != '*' && self.peek_next() != '/' && !self.is_at_end() {
                            if self.peek() == '\n' {
                                self.line += 1;
                            }
                            self.advance();
                        }
                        self.advance();
                        self.advance();
                    }
                    _ => self.add_token(TokenType::Slash),
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => {
                self.line += 1;
            }
            '"' => self.string()?,
            '0'..='9' => self.number(),
            c if is_alpha(c) => self.identifier(),
            _ => {
                return Err(LoxError::Parsing {
                    line: self.line,
                    whence: std::ascii::escape_default(c as u8).to_string(),
                    message: "Unexpected character".to_string(),
                });
            }
        }

        Ok(())
    }

    fn advance(&mut self) -> char {
        let next = self.source.as_bytes()[self.current] as char;
        self.current += 1;
        next
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, crate::object::Literal::Null)
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: crate::object::Literal) {
        let text = &self.source[self.start..self.current];
        self.tokens
            .push(Token::new(token_type, text, literal, self.line))
    }

    fn advance_if_is(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.as_bytes()[self.current] as char != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.as_bytes()[self.current] as char
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source.as_bytes()[self.current + 1] as char
    }

    fn string(&mut self) -> Result<()> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(LoxError::Parsing {
                line: self.line,
                whence: "EOF".to_string(),
                message: "Unterminated string.".to_string(),
            });
        }

        // The closing "
        self.advance();

        let val = &self.source[self.start + 1..self.current - 1];
        self.add_token_with_literal(
            TokenType::String,
            crate::object::Literal::String(val.to_string()),
        );
        Ok(())
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for a fractional part
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume the '.'
            self.advance();
        }

        while self.peek().is_ascii_digit() {
            self.advance();
        }

        let as_float: f64 = self.source[self.start..self.current]
            .parse::<f64>()
            .expect("Better be a number");
        self.add_token_with_literal(
            TokenType::Number,
            crate::object::Literal::Number(OrderedFloat(as_float)),
        )
    }

    fn identifier(&mut self) {
        while is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let token_type = match TokenType::try_from_identifier(text) {
            Some(tt) => tt,
            None => TokenType::Identifier,
        };

        self.add_token(token_type);
    }
}
fn is_alpha(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_alphanumeric(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
