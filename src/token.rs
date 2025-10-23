use crate::{object::Literal, token_type::TokenType};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}

#[automatically_derived]
impl ::core::fmt::Debug for Token {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.typ {
            TokenType::Greater => write!(f, ">"),
            TokenType::Less => write!(f, "<"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Plus => write!(f, "+"),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::Identifier => write!(f, "{}{{ {:?} }}[L{}]", self.lexeme, self.literal, self.line),
            _ => f
                .debug_struct("Token")
                .field("typ", &self.typ)
                .field("lexeme", &self.lexeme)
                .field("literal", &self.literal)
                .field("line", &&self.line)
                .finish(),
        }
    }
}

impl Token {
    pub fn new(typ: TokenType, lexeme: &str, literal: Literal, line: usize) -> Token {
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
