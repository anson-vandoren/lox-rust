use crate::{object::Object, token_type::TokenType};

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub literal: Object,
    pub line: usize,
}

impl std::hash::Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.typ.hash(state);
        self.lexeme.hash(state);
        // at least the same variant
        core::mem::discriminant::<Object>(&self.literal).hash(state);
        self.line.hash(state);
    }
}

impl Eq for Token {}

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
