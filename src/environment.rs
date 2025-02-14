use std::collections::{hash_map::Entry, HashMap};

use crate::{object::Object, token::Token, LoxError, Result};

pub struct Environment {
    values: HashMap<String, Object>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Environment {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_enclosing(self) -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: Some(Box::new(self)),
        }
    }

    pub fn into_outer(self) -> Option<Environment> {
        if let Some(outer) = self.enclosing {
            Some(*outer)
        } else {
            None
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: Token, value: Object) -> Result<()> {
        match self.values.entry(name.lexeme.clone()) {
            Entry::Vacant(_) => {
                if let Some(ref mut outer) = self.enclosing {
                    outer.assign(name, value)
                } else {
                    Err(LoxError::Runtime {
                        expected: format!("Variable '{}' to be defined.", name.lexeme),
                        found: "undefined".to_string(),
                        token: name,
                    })
                }
            }
            Entry::Occupied(mut entry) => {
                entry.insert(value);
                Ok(())
            }
        }
    }

    pub fn get(&self, name: Token) -> Result<Object> {
        match self.values.get(&name.lexeme) {
            Some(val) => Ok(val.clone()),
            None => {
                if let Some(outer) = &self.enclosing {
                    outer.get(name)
                } else {
                    Err(LoxError::Runtime {
                        expected: "Defined variable name".to_string(),
                        found: name.lexeme.clone(),
                        token: name,
                    })
                }
            }
        }
    }
}
