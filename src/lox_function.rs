use crate::{
    LoxError,
    interpreter::{
        Interpreter,
        environment::{Environment, RcCell},
    },
    lox_callable::LoxCallable,
    object::{Literal, Object, ObjectRuntimeError},
    stmt::Function,
};

#[derive(Clone, Debug)]
pub struct LoxFunction {
    declaration: Function,
    closure: RcCell<Environment>,
}

impl LoxFunction {
    pub fn new(declaration: Function, closure: RcCell<Environment>) -> Self {
        Self { declaration, closure }
    }
}

impl std::fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object, ObjectRuntimeError> {
        let mut environment = Environment::with_enclosing(self.closure.clone());
        arguments.into_iter().enumerate().for_each(|(i, arg)| {
            environment.define(self.declaration.params[i].lexeme.clone(), arg);
        });

        match interpreter.execute_block(&self.declaration.body, environment).map_err(|e| match e {
            LoxError::Return { value } => Ok(value),
            other => Err(other),
        }) {
            Ok(()) => Ok(Object::Literal(Literal::Null)),
            Err(Ok(value)) => Ok(value),
            Err(Err(e)) => Err(match e {
                LoxError::Runtime { found, expected, token: _ } => ObjectRuntimeError { found, expected },
                _ => ObjectRuntimeError {
                    found: "unknown".to_string(),
                    expected: "unknown".to_string(),
                },
            }),
        }
    }

    fn arity(&self) -> u8 {
        self.declaration.params.len() as u8
    }

    fn name(&self) -> &str {
        &self.declaration.name.lexeme
    }
}
