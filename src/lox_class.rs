use std::collections::HashMap;

use crate::{
    LoxError, interpreter::Interpreter, lox_callable::LoxCallable, lox_function::LoxFunction, lox_instance::LoxInstance, object::Object,
};

#[derive(Clone, Debug)]
pub struct LoxClass {
    pub name: String,
    pub methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    pub fn new<T>(name: T, methods: HashMap<String, LoxFunction>) -> Self
    where
        T: Into<String>,
    {
        Self {
            name: name.into(),
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<LoxFunction> {
        self.methods.get(name).cloned()
    }
}

impl LoxCallable for LoxClass {
    fn call(&self, _interpreter: &mut Interpreter, _arguments: Vec<Object>) -> Result<Object, LoxError> {
        Ok(Object::Instance(LoxInstance::new(self.clone())))
    }

    fn arity(&self) -> u8 {
        0
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl std::fmt::Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
