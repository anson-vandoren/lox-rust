use std::collections::HashMap;

use snafu::whatever;
use tracing::trace;

use super::Interpreter;
use crate::{
    Result,
    expr::Expr,
    stmt::{self, Stmt},
    token::Token,
};

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    curr_fn: FunctionType,
}

#[derive(Clone, Copy)]
enum FunctionType {
    None,
    Function,
    Method,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: vec![],
            curr_fn: FunctionType::None,
        }
    }

    pub fn resolve_all(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        let mut had_error = false;
        for statement in statements {
            if let Err(err) = self.resolve_stmt(statement).inspect_err(|_| had_error = true) {
                // Keep going with the analysis, error at the end
                eprintln!("{}", err);
            }
        }

        if had_error {
            whatever!("One or more errors during static analysis")
        }
        Ok(())
    }
}

// Expressions
impl Resolver<'_> {
    fn resolve_expr(&mut self, expr: &Expr) -> Result<()> {
        trace!(?expr, "Resolving expression");
        match expr {
            Expr::Variable(var) => {
                trace!("Expr::Variable {}", &var.name);
                if let Some(peeked) = self.scopes.last() {
                    if peeked.get(&var.name.lexeme) == Option::from(&false) {
                        whatever!("Cannot read a local variable in its own initializer.");
                    }
                }

                self.resolve_local(&var.name)?;
            }
            Expr::Assign(assign) => {
                trace!("Expr::Assign {}", &assign.name);
                self.resolve_expr(&assign.value)?;
                self.resolve_local(&assign.name)?;
            }
            Expr::Binary(binary) => {
                trace!(?expr, "Expr::Binary");
                self.resolve_expr(&binary.left)?;
                self.resolve_expr(&binary.right)?;
            }
            Expr::Call(call) => {
                self.resolve_expr(&call.callee)?;
                for arg in call.arguments.iter() {
                    self.resolve_expr(arg)?;
                }
            }
            Expr::Get(get) => {
                self.resolve_expr(&get.object)?;
            }
            Expr::Grouping(group) => {
                self.resolve_expr(&group.expression)?;
            }
            Expr::Literal(_) => (),
            Expr::Logical(logic) => {
                self.resolve_expr(&logic.left)?;
                self.resolve_expr(&logic.right)?;
            }
            Expr::Unary(unary) => self.resolve_expr(&unary.right)?,
            Expr::Set(set) => {
                self.resolve_expr(&set.value)?;
                self.resolve_expr(&set.object)?;
            }
        }
        trace!(?expr, "Exited expression");
        Ok(())
    }
}

// Statements
impl Resolver<'_> {
    fn resolve_stmt(&mut self, statement: &Stmt) -> Result<()> {
        trace!(?statement, "Resolving statement");
        match statement {
            Stmt::Var(var) => {
                self.declare(&var.name.lexeme)?;
                if let Some(initializer) = &var.initializer {
                    trace!(?initializer, "had initializer");
                    self.resolve_expr(initializer)?;
                }
                self.define(&var.name.lexeme)?;
            }
            Stmt::Function(func) => {
                self.declare(&func.name.lexeme)?;
                self.define(&func.name.lexeme)?;

                self.resolve_func(func, FunctionType::Function)?;
            }
            Stmt::Expression(expr) => self.resolve_expr(&expr.expression)?,
            Stmt::If(stmt) => {
                self.resolve_expr(&stmt.condition)?;
                self.resolve_stmt(&stmt.then_branch)?;
                if let Some(else_branch) = &stmt.else_branch {
                    self.resolve_stmt(else_branch)?;
                }
            }
            Stmt::Print(stmt) => {
                self.resolve_expr(&stmt.expression)?;
            }
            Stmt::Return(stmt) => {
                if let FunctionType::None = self.curr_fn {
                    whatever!("Cannot return from top-level code. {:?}", stmt.keyword)
                }
                if let Some(val) = &stmt.value {
                    self.resolve_expr(val)?;
                }
            }
            Stmt::While(stmt) => {
                self.resolve_expr(&stmt.condition)?;
                self.resolve_stmt(&stmt.body)?;
            }
            Stmt::Block(block) => {
                self.begin_scope();
                self.resolve_all(&block.statements)?;
                self.end_scope()?;
            }
            Stmt::Class(stmt) => {
                self.declare(&stmt.name.lexeme)?;
                self.define(&stmt.name.lexeme)?;

                for method in stmt.methods.iter() {
                    let declaration = FunctionType::Method;
                    self.resolve_func(method, declaration)?;
                }
            }
        }
        trace!(?statement, "Finished resolving statement");
        Ok(())
    }

    fn resolve_func(&mut self, func: &stmt::Function, typ: FunctionType) -> Result<()> {
        let enclosing_fn = self.curr_fn;
        self.curr_fn = typ;
        self.begin_scope();
        for param in func.params.iter() {
            self.declare(&param.lexeme)?;
            self.define(&param.lexeme)?;
        }
        self.resolve_all(&func.body)?;
        self.end_scope()?;
        self.curr_fn = enclosing_fn;
        Ok(())
    }
}

// Helpers
impl Resolver<'_> {
    fn begin_scope(&mut self) {
        trace!(len = self.scopes.len(), "Beginning scope");
        self.scopes.push(HashMap::new());
        trace!(len = self.scopes.len(), "Done beginning scope");
    }

    fn end_scope(&mut self) -> Result<()> {
        trace!(len = self.scopes.len(), "Ending scope");
        if self.scopes.pop().is_none() {
            whatever!("Ended a scope when there was no stack")
        }
        trace!(len = self.scopes.len(), "Done ending scope");
        Ok(())
    }

    fn declare(&mut self, name: &str) -> Result<()> {
        trace!(name, len = self.scopes.len(), ">> Declaring");
        if self.scopes.is_empty() {
            trace!("<< Declaring, no scopes");
            return Ok(());
        }

        if let Some(peeked) = self.scopes.last_mut() {
            if peeked.contains_key(name) {
                whatever!("'{name}' is already defined in this scope");
            }
            peeked.insert(name.to_string(), false);
        } else {
            whatever!("Should have a scope by 'declare'")
        }
        trace!(name, len = self.scopes.len(), "<< Declaring, into parent");
        Ok(())
    }

    fn define(&mut self, name: &str) -> Result<()> {
        trace!(name, len = self.scopes.len(), ">> Resolver.define()");
        if self.scopes.is_empty() {
            trace!("<< Resolver.define(), no scope");
            return Ok(());
        }

        if let Some(peeked) = self.scopes.last_mut() {
            peeked.insert(name.to_string(), true);
        } else {
            whatever!("Didn't have initial scope in define")
        }
        trace!(name, len = self.scopes.len(), "<< Resolver.define(), into parent scope");
        Ok(())
    }

    fn resolve_local(&mut self, token: &Token) -> Result<()> {
        trace!(?token, len = self.scopes.len(), "Resolving local");
        let top = self.scopes.len();
        for i in (0..top).rev() {
            if self.scopes[i].contains_key(&token.lexeme) {
                let depth = (self.scopes.len() - 1 - i).try_into();
                let depth = whatever!(depth, "Depth larger than u8");
                self.interpreter.resolve(token, depth);
                return Ok(());
            }
        }

        Ok(())
    }
}
