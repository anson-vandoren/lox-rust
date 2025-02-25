use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    rc::Rc,
};

use tracing::trace;

use crate::{LoxError, Result, object::Object, token::Token};
pub(crate) type RcCell<T> = Rc<RefCell<T>>;

#[derive(Clone, Debug, Default)]
pub struct Environment {
    pub values: HashMap<String, Object>,
    pub enclosing: Option<RcCell<Environment>>,
}

impl Environment {
    pub fn new() -> Environment {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing(enclosing: RcCell<Environment>) -> Environment {
        let values = enclosing.as_ref();
        trace!(new_top=?values, "with_enclosing");
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
                if let Some(ref outer) = self.enclosing {
                    let mut outer = outer.as_ref().borrow_mut();
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
        if *distance == 0 {
            self.values.insert(name.to_string(), value);
        } else {
            let env = ancestor(self.enclosing.clone().unwrap(), distance);
            let mut env = env.as_ref().borrow_mut();
            env.values.insert(name.to_string(), value);
        }
        Ok(())
    }

    pub fn get(&self, name: &Token) -> Result<Object> {
        trace!(?name, values = ?self.values, ">> Environment.get()");
        match self.values.get(&name.lexeme) {
            Some(val) => Ok(val.clone()),
            None => {
                if let Some(outer) = &self.enclosing {
                    let outer = outer.as_ref().borrow();
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
        if *distance == 0 {
            Ok(self.values.get(key).cloned().ok_or(LoxError::Internal {
                message: format!("Expected variable '{key}' at distance {distance}"),
            })?)
        } else {
            let env = ancestor(self.enclosing.clone().unwrap(), distance);
            Ok(env.borrow().values.get(key).cloned().ok_or(LoxError::Internal {
                message: format!("Expected variable '{key}' at distance {distance}"),
            })?)
        }
    }
}

fn ancestor(env: RcCell<Environment>, distance: &u8) -> RcCell<Environment> {
    let mut env = env;
    trace!(values = ?env.as_ref().borrow().values, "env top-level");
    for i in 0_u8..*distance {
        let next = {
            let cur_borrow = env.borrow();
            cur_borrow.enclosing.as_ref().unwrap().clone()
        };

        env = next;
        let dist = i + 1;
        trace!(values = ?env.as_ref().borrow().values, "env at depth {dist}");
    }
    trace!(values=?env.as_ref().borrow().values, distance, "Chosen ancestor env");
    env
}
