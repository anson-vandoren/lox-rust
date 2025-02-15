use crate::{
    interpreter::Interpreter,
    object::{Object, ObjectRuntimeError},
};

pub trait LoxCallable {
    fn call(&self, interpreter: Interpreter, arguments: Vec<Object>) -> Result<Object, ObjectRuntimeError>;
    fn arity(&self) -> u8;
    fn name(&self) -> &'static str;
}
