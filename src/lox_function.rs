use std::{cell::RefCell, rc::Rc};

use tracing::{instrument, trace};

use crate::{
    LoxError,
    interpreter::{
        Interpreter,
        environment::{Environment, RcCell},
    },
    lox_callable::LoxCallable,
    lox_instance::LoxInstance,
    object::{Literal, Object},
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

    #[instrument(skip(self, instance))]
    pub(crate) fn bind(&self, instance: &LoxInstance) -> Result<Object, LoxError> {
        let mut environment = Environment::with_parent(self.closure.clone());
        environment.define("this".into(), Object::Instance(instance.clone()));
        trace!(vals = ?environment.values, "After binding this");

        let environment = Rc::new(RefCell::new(environment));

        Ok(Object::Callable(Rc::new(LoxFunction::new(self.declaration.clone(), environment))))
    }
}

impl std::fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}

impl LoxCallable for LoxFunction {
    #[instrument(skip(self, interpreter), err)]
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object, LoxError> {
        let mut environment = Environment::with_parent(self.closure.clone());
        trace!(?environment, "Initial enclosed environment");
        arguments.into_iter().enumerate().for_each(|(i, arg)| {
            let name = self.declaration.params[i].lexeme.clone();
            trace!(name, ?arg, "Defining additional argument in environment");
            environment.define(name, arg);
        });
        trace!(?environment, "Environment for call");

        match interpreter.execute_block(&self.declaration.body, environment).map_err(|e| match e {
            LoxError::Return { value } => Ok(value),
            other => Err(other),
        }) {
            Ok(()) => Ok(Object::Literal(Literal::Null)),
            Err(Ok(value)) => Ok(value),
            Err(Err(e)) => Err(e),
        }
    }

    fn arity(&self) -> u8 {
        self.declaration.params.len() as u8
    }

    fn name(&self) -> &str {
        &self.declaration.name.lexeme
    }
}
