use crate::{interpreter::Interpreter, object::Object, Result};

pub trait LoxCallable {
    fn call(&self, interpreter: Interpreter, arguments: Vec<Object>) -> Result<Object>;
    fn arity(&self) -> u8;
}
