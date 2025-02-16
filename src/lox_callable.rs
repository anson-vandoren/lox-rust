use crate::{
    interpreter::Interpreter,
    object::{Object, ObjectRuntimeError},
};

pub trait LoxCallable: std::fmt::Display {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object, ObjectRuntimeError>;
    fn arity(&self) -> u8;
    fn name(&self) -> &str;
}
