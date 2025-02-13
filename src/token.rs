use crate::{object::Object, token_type::TokenType};

#[derive(Clone, Debug)]
pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub literal: Object,
    pub line: usize,
}

impl Token {
    pub fn new(typ: TokenType, lexeme: &str, literal: Object, line: usize) -> Token {
        Token {
            typ,
            lexeme: lexeme.to_string(),
            literal,
            line,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {} {:?}", self.typ, self.lexeme, self.literal)
    }
}
