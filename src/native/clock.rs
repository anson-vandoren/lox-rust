use std::time::SystemTime;

use ordered_float::OrderedFloat;

use crate::{LoxError, interpreter::Interpreter, lox_callable::LoxCallable, object::Object};

pub struct LoxClock {}

impl std::fmt::Display for LoxClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native function>")
    }
}

impl LoxCallable for LoxClock {
    fn call(&self, _interpreter: &mut Interpreter, _arguments: Vec<Object>) -> Result<Object, LoxError> {
        Ok(Object::from(OrderedFloat(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Unix Epoch was a long damn time ago")
                .as_secs() as f64,
        )))
    }

    fn arity(&self) -> u8 {
        0
    }

    fn name(&self) -> &'static str {
        "system_clock"
    }
}
