use crate::{
    interpreter::Interpreter,
    lox_callable::LoxCallable,
    lox_instance::LoxInstance,
    object::{Object, ObjectRuntimeError},
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LoxClass {
    pub name: String,
}

impl LoxClass {
    pub fn new<T>(name: T) -> Self
    where
        T: Into<String>,
    {
        Self { name: name.into() }
    }
}

impl LoxCallable for LoxClass {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object, ObjectRuntimeError> {
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
