use crate::{
    interpreter::{environment::Environment, Interpreter},
    lox_callable::LoxCallable,
    object::{Object, ObjectRuntimeError},
    stmt::{Function, Stmt},
    LoxError,
};

pub struct LoxFunction {
    declaration: Function,
}

impl LoxFunction {
    pub fn new(declaration: Stmt) -> Result<Self, ObjectRuntimeError> {
        if let Stmt::Function(decl) = declaration {
            Ok(Self { declaration: decl })
        } else {
            Err(ObjectRuntimeError {
                found: format!("{:?}", declaration),
                expected: "A Function Statement.".to_string(),
            })
        }
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object, ObjectRuntimeError> {
        let mut environment = Environment::with_enclosing(Box::new(interpreter.globals.clone()));
        arguments.into_iter().enumerate().for_each(|(i, arg)| {
            environment.define(self.declaration.params[i].lexeme.clone(), arg);
        });

        match interpreter.execute_block(&self.declaration.body, environment).map_err(|e| match e {
            LoxError::Return { value } => Ok(value),
            other => Err(other),
        }) {
            Ok(()) => Ok(Object::Null),
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

impl std::fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}

impl PartialEq for LoxFunction {
    fn eq(&self, other: &Self) -> bool {
        self.declaration.params.len() == other.declaration.params.len() && self.declaration.name.lexeme == other.declaration.name.lexeme
    }
}
