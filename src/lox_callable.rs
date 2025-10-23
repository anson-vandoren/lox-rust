use crate::{LoxError, interpreter::Interpreter, object::Object};

pub trait LoxCallable: std::fmt::Display {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object, LoxError>;
    fn arity(&self) -> u8;
    fn name(&self) -> &str;
}
