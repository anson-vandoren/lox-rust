use std::collections::{hash_map::Entry, HashMap};

use crate::{object::Object, token::Token, LoxError, Result};

#[derive(Clone)]
pub struct Environment {
    pub values: HashMap<String, Object>,
    pub enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Environment {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing(enclosing: Box<Environment>) -> Environment {
        Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<()> {
        match self.values.entry(name.lexeme.clone()) {
            Entry::Vacant(_) => {
                if let Some(ref mut outer) = self.enclosing {
                    outer.assign(name, value)
                } else {
                    Err(LoxError::Runtime {
                        expected: format!("Variable '{}' to be defined.", name.lexeme),
                        found: "undefined".to_string(),
                        token: name.clone(),
                    })
                }
            }
            Entry::Occupied(mut entry) => {
                entry.insert(value);
                Ok(())
            }
        }
    }

    pub fn get(&self, name: &Token) -> Result<Object> {
        match self.values.get(&name.lexeme) {
            Some(val) => Ok(val.clone()),
            None => {
                if let Some(outer) = &self.enclosing {
                    outer.get(name)
                } else {
                    Err(LoxError::Runtime {
                        expected: "Defined variable name".to_string(),
                        found: name.lexeme.clone(), // TODO: clone
                        token: name.clone(),        // TODO: clone
                    })
                }
            }
        }
    }
}
