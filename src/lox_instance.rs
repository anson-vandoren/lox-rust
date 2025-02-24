use std::{collections::HashMap, rc::Rc};

use tracing::trace;

use crate::{LoxError, lox_class::LoxClass, object::Object, token::Token};

#[derive(Clone, Debug)]
pub struct LoxInstance {
    class: LoxClass,
    fields: HashMap<String, Object>,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Object, LoxError> {
        trace!(fields = ?self.fields, ?name, class = ?self.class, "LoxInstance.get()");
        let field = self.fields.get(&name.lexeme).cloned();
        if let Some(field) = field {
            return Ok(field);
        }

        let method = self.class.find_method(&name.lexeme);
        if let Some(method) = method {
            return Ok(Object::Callable(Rc::new(method)));
        }

        Err(LoxError::Runtime {
            expected: format!("method or field named {}", name.lexeme),
            found: "no such method or field".into(),
            token: name.clone(),
        })
    }

    pub fn set(&mut self, name: Token, value: Object) {
        trace!(fields = ?self.fields, ?name, class = ?self.class, value = ?value, "LoxInstance.set()");
        self.fields.insert(name.lexeme, value);
    }
}

impl std::fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance - {:?}", self.class, self.fields)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::{assert_eq, assert_matches};

    use super::*;
    use crate::{
        object::{Literal, Object},
        token_type::TokenType,
    };

    #[test]
    fn gets_and_sets() {
        let token = Token::new(TokenType::Identifier, "foo", Literal::Null, 0);
        let obj = Object::Literal(Literal::from(42));
        let mut instance = LoxInstance::new(LoxClass::new("fake", HashMap::new()));

        instance.set(token.clone(), obj.clone());
        let got = instance.get(&token).unwrap();
        assert_eq!(got, obj);
    }

    #[test]
    fn only_cares_about_lexeme() {
        let token = Token::new(TokenType::Identifier, "foo", Literal::Null, 0);
        let obj = Object::Literal(Literal::from(42));
        let mut instance = LoxInstance::new(LoxClass::new("fake", HashMap::new()));
        instance.set(token.clone(), obj.clone());

        let other_token = Token::new(TokenType::LeftParen, "foo", Literal::from(666), 42);
        let got = instance.get(&other_token).unwrap();
        assert_eq!(got, obj);
    }

    #[test]
    fn errors_when_missing() {
        let token = Token::new(TokenType::Identifier, "foo", Literal::Null, 0);
        let instance = LoxInstance::new(LoxClass::new("fake", HashMap::new()));

        let got = instance.get(&token);
        assert_matches!(got, Err(LoxError::Internal { .. }));
    }

    #[test]
    fn replaces_when_setting_over() {
        let token = Token::new(TokenType::Identifier, "foo", Literal::Null, 0);
        let obj = Object::Literal(Literal::from(42));
        let mut instance = LoxInstance::new(LoxClass::new("fake", HashMap::new()));
        instance.set(token.clone(), obj.clone());

        let other_obj = Object::Literal(Literal::from("42"));
        instance.set(token.clone(), other_obj.clone());
        let got = instance.get(&token).unwrap();
        assert_eq!(got, other_obj);
    }
}
