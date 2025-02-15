use crate::{Result, interpreter::Interpreter, object::Object};

pub trait LoxCallable {
    fn call(&self, interpreter: Interpreter, arguments: Vec<Object>) -> Result<Object>;
    fn arity(&self) -> u8;
}
