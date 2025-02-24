use crate::{
    interpreter::Interpreter,
    lox_callable::LoxCallable,
    object::{Literal, Object, ObjectRuntimeError},
};

pub struct LoxAssertEq {}

impl std::fmt::Display for LoxAssertEq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native function>")
    }
}

impl LoxCallable for LoxAssertEq {
    fn call(&self, _interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object, ObjectRuntimeError> {
        if arguments.len() != 2 {
            return Err(ObjectRuntimeError {
                found: format!("{} args", arguments.len()),
                expected: "2 arguments".into(),
            });
        }
        let first = arguments.first().expect("already checked");
        let second = arguments.get(1).expect("already checked");
        if first == second {
            Ok(Object::Literal(Literal::Null))
        } else {
            Err(ObjectRuntimeError {
                found: format!("{} != {}", first, second),
                expected: format!("{} == {}", first, second),
            })
        }
    }

    fn arity(&self) -> u8 {
        2
    }

    fn name(&self) -> &'static str {
        "assert"
    }
}
