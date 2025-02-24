use std::collections::{HashMap, hash_map::Entry};

use tracing::trace;

use crate::{LoxError, Result, object::Object, token::Token};

#[derive(Clone, Debug, Default)]
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
        trace!(new_top=?enclosing.values, "with_enclosing");
        Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        trace!(?name, ?value, current=?self.values, "defining");
        self.values.insert(name, value);
        trace!(current=?self.values, "done defining");
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<()> {
        trace!(?name, ?value, values = ?self.values, ">> assign()");
        match self.values.entry(name.lexeme.clone()) {
            Entry::Vacant(_) => {
                if let Some(ref mut outer) = self.enclosing {
                    let res = outer.assign(name, value);
                    trace!(values = ?self.values, "<< assign(), vacant");
                    res
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
                trace!(values = ?self.values, "<< assign(), occupied");
                Ok(())
            }
        }
    }

    pub fn assign_at(&mut self, distance: &u8, name: &str, value: Object) -> Result<()> {
        trace!(distance, ?name, ?value, "Assigning to env ancestor");
        self.ancestor(distance).values.insert(name.to_string(), value);
        Ok(())
    }

    pub fn get(&self, name: &Token) -> Result<Object> {
        trace!(?name, values = ?self.values, ">> Environment.get()");
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

    pub fn get_at(&mut self, distance: &u8, key: &str) -> Result<Object> {
        trace!(distance, key, "Get at");
        self.ancestor(distance).values.get(key).cloned().ok_or(LoxError::Internal {
            message: format!("Expected variable '{key}' at distance {distance}"),
        })
    }

    fn ancestor(&mut self, distance: &u8) -> &mut Environment {
        let mut env = self;
        trace!(values = ?env.values, "env top-level");
        for i in 0_u8..*distance {
            env = &mut *env.enclosing.as_mut().expect("Should have had an enclosing scope");
            let dist = i + 1;
            trace!(values = ?env.values, "env at depth {dist}");
        }
        trace!(values=?env.values, distance, "Chosen ancestor env");
        env
    }
}
