use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    rc::Rc,
};

use tracing::{instrument, trace};

use crate::{LoxError, Result, object::Object, token::Token};
pub(crate) type RcCell<T> = Rc<RefCell<T>>;

#[derive(Clone, Debug, Default)]
pub struct Environment {
    pub values: HashMap<String, Object>,
    pub parent: Option<RcCell<Environment>>,
}

impl Environment {
    pub fn depth(&self) -> i32 {
        let mut depth = 0;
        let mut parent = self.parent.clone();
        while let Some(next) = parent {
            depth += 1;
            parent = next.borrow().parent.clone();
        }
        depth
    }
}

impl Environment {
    pub fn new() -> Environment {
        Self {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(enclosing: RcCell<Environment>) -> Environment {
        let values = enclosing.as_ref();
        let new_depth = enclosing.borrow().depth() + 1;
        trace!(new_depth, parent= ?values, "with_parent");
        Self {
            values: HashMap::new(),
            parent: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        let at_depth = self.depth();
        trace!(at_depth, ?name, ?value, current=?self.values, "defining");
        self.values.insert(name, value);
        trace!(current=?self.values, "done defining");
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<()> {
        trace!(?name, ?value, values = ?self.values, ">> assign()");
        match self.values.entry(name.lexeme.clone()) {
            Entry::Vacant(_) => {
                if let Some(ref outer) = self.parent {
                    let mut outer = outer.as_ref().borrow_mut();
                    let res = outer.assign(name, value);
                    trace!(values = ?self.values, "<< assign(), vacant");
                    res
                } else {
                    Err(LoxError::Runtime {
                        expected: format!("Variable '{}' to be defined.", name.lexeme),
                        found: "undefined".to_string(),
                        line: Some(name.line),
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
            let env = ancestor(self.parent.clone().unwrap(), distance - 1);
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
                if let Some(outer) = &self.parent {
                    let outer = outer.as_ref().borrow();
                    outer.get(name)
                } else {
                    Err(LoxError::Runtime {
                        expected: "Defined variable name".to_string(),
                        found: name.lexeme.clone(),
                        line: Some(name.line),
                    })
                }
            }
        }
    }

    #[instrument(skip(self))]
    pub fn get_at(&mut self, distance: &u8, key: &str) -> Result<Object> {
        trace!(distance, key, "Get at");
        if *distance == 0 {
            Ok(self.values.get(key).cloned().ok_or(LoxError::Internal {
                message: format!("Expected variable '{key}' at distance {distance}"),
            })?)
        } else {
            let parent = self.parent.clone().ok_or(LoxError::Internal {
                message: "Expected a parent".to_string(),
            })?;
            let env = ancestor(parent, distance - 1);
            Ok(env.borrow().values.get(key).cloned().ok_or(LoxError::Internal {
                message: format!("Expected variable '{key}' at distance {distance}"),
            })?)
        }
    }
}

fn ancestor(env: RcCell<Environment>, distance: u8) -> RcCell<Environment> {
    trace!(">>ancestor()");
    let mut env = env;
    trace!(distance, env=?env.as_ref().borrow().values, "env top-level");
    for i in 0_u8..distance {
        let next = {
            let cur_borrow = env.borrow();
            cur_borrow.parent.as_ref().unwrap().clone()
        };

        env = next;
        let dist = i + 1;
        trace!("env at depth {dist}: {:#?}", env.as_ref().borrow().values);
    }
    trace!(distance, env=?env.as_ref().borrow().values, "Chosen ancestor env");
    env
}
