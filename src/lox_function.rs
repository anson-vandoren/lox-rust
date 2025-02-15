use crate::{
    object::ObjectRuntimeError,
    stmt::{Function, Stmt},
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
