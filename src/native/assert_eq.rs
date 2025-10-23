use crate::{
    LoxError,
    interpreter::Interpreter,
    lox_callable::LoxCallable,
    object::{Literal, Object},
};

pub struct LoxAssertEq {}

impl std::fmt::Display for LoxAssertEq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native function>")
    }
}

impl LoxCallable for LoxAssertEq {
    fn call(&self, _interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object, LoxError> {
        if arguments.len() != 2 {
            return Err(LoxError::Runtime {
                found: format!("{} args", arguments.len()),
                expected: "2 arguments".into(),
                line: None,
            });
        }
        let first = arguments.first().expect("already checked");
        let second = arguments.get(1).expect("already checked");
        if first == second {
            Ok(Object::Literal(Literal::Null))
        } else {
            Err(LoxError::Runtime {
                found: format!("{} != {}", first, second),
                expected: format!("{} == {}", first, second),
                line: None,
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
