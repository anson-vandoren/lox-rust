use std::{collections::HashMap, ops::ControlFlow};

use super::Interpreter;
use crate::{
    expr::Expr,
    stmt::{self, Stmt},
};

pub struct Resolver<'a> {
    interpreter: &'a Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
        }
    }

    pub fn resolve_all(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }
}

// Expressions
impl<'a> Resolver<'_> {
    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Variable(var) => {
                if let Some(peeked) = self.scopes.last() {
                    if peeked.get(&var.name.lexeme) == Option::from(&false) {
                        panic!("Cannot read a local variable in its own initializer.");
                    }
                }

                self.resolve_local(expr, &var.name.lexeme)
            }
            Expr::Assign(assign) => {
                self.resolve_expr(&assign.value);
                self.resolve_local(expr, &assign.name.lexeme);
            }
            Expr::Binary(binary) => {
                self.resolve_expr(&binary.left);
                self.resolve_expr(&binary.right);
            }
            Expr::Call(call) => {
                self.resolve_expr(&call.callee);
                call.arguments.iter().for_each(|arg| self.resolve_expr(arg));
            }
            Expr::Grouping(group) => {
                self.resolve_expr(&group.expression);
            }
            Expr::Literal(_) => (),
            Expr::Logical(logic) => {
                self.resolve_expr(&logic.left);
                self.resolve_expr(&logic.right);
            }
            Expr::Unary(unary) => self.resolve_expr(&unary.right),
        }
    }
}

// Statements
impl<'a> Resolver<'_> {
    fn resolve_stmt(&mut self, statement: Stmt) {
        match statement {
            Stmt::Var(var) => {
                self.declare(&var.name.lexeme);
                if let Some(initializer) = var.initializer {
                    self.resolve_expr(&initializer)
                }
                self.define(&var.name.lexeme);
            }
            Stmt::Function(func) => {
                self.declare(&func.name.lexeme);
                self.define(&func.name.lexeme);

                self.resolve_func(func);
            }
            Stmt::Expression(expr) => self.resolve_expr(&expr.expression),
            Stmt::If(stmt) => {
                self.resolve_expr(&stmt.condition);
                self.resolve_stmt(*stmt.then_branch);
                if let Some(else_branch) = stmt.else_branch {
                    self.resolve_stmt(*else_branch)
                }
            }
            Stmt::Print(stmt) => {
                self.resolve_expr(&stmt.expression);
            }
            Stmt::Return(stmt) => {
                if let Some(val) = stmt.value {
                    self.resolve_expr(&val);
                }
            }
            Stmt::While(stmt) => {
                self.resolve_expr(&stmt.condition);
                self.resolve_stmt(*stmt.body);
            }
            Stmt::Block(block) => {
                self.begin_scope();
                self.resolve_all(block.statements);
                self.end_scope();
            }
        }
    }

    fn resolve_func(&mut self, func: stmt::Function) {
        self.begin_scope();
        for param in func.params {
            self.declare(&param.lexeme);
            self.define(&param.lexeme);
        }
        self.resolve_all(func.body);
        self.end_scope();
    }
}

// Helpers
impl<'a> Resolver<'_> {
    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    fn end_scope(&mut self) {
        self.scopes.pop().unwrap();
    }

    fn declare(&mut self, name: &str) {
        if self.scopes.is_empty() {
            return;
        }

        let scope: &mut HashMap<_, _> = self.scopes.last_mut().expect("Should have a scope by 'declare'");
        if scope.contains_key(name) {
            panic!("'{name}' is already defined in this scope");
        }
        scope.insert(name.to_string(), false);
    }

    fn define(&mut self, name: &str) {
        if self.scopes.is_empty() {
            return;
        }

        self.scopes
            .last_mut()
            .expect("Should have a scope by 'define'")
            .insert(name.to_string(), true);
    }

    fn resolve_local(&self, expr: &Expr, name: &str) {
        let _ = self.scopes.iter().enumerate().rev().try_for_each(|(i, scope)| {
            if scope.contains_key(name) {
                let depth: u8 = (self.scopes.len() - 1 - i).try_into().expect("Depth larger than u8");
                self.interpreter.resolve(expr, depth);
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });
    }
}
