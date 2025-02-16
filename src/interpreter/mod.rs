pub mod environment;
pub mod resolver;

use std::{collections::HashMap, rc::Rc};

use environment::Environment;
use ordered_float::OrderedFloat;
use tracing::{instrument, trace};

use super::{LoxError, Result};
use crate::{
    expr::{self, Expr},
    lox_callable::LoxCallable as _,
    lox_function::LoxFunction,
    native::clock::LoxClock,
    object::Object,
    stmt::{self, Stmt},
    token_type::TokenType,
};

pub struct Interpreter {
    environment: Box<Environment>,
    pub globals: Environment,
    locals: HashMap<Expr, u8>,
}

impl Default for Interpreter {
    fn default() -> Self {
        let bare = Box::new(Environment::new());
        Self {
            environment: bare.clone(),
            globals: *bare,
            locals: HashMap::new(),
        }
    }
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut globals = Environment::new();
        globals.define("clock".to_string(), Object::Callable(Rc::new(LoxClock {})));
        Self {
            environment: Box::new(globals.clone()),
            globals,
            locals: HashMap::new(),
        }
    }

    #[instrument(skip(self, statements), err, ret, level = "trace")]
    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<()> {
        for statement in statements {
            self.execute(&statement)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        trace!(?stmt, "Excuting statement");
        match stmt {
            Stmt::Print(stmt) => self.execute_print_stmt(stmt),
            Stmt::Block(stmt) => self.execute_block(&stmt.statements, Environment::new()),
            Stmt::Expression(stmt) => self.evaluate(&stmt.expression).map(|_| ()),
            Stmt::Var(stmt) => self.execute_var_stmt(stmt),
            Stmt::If(stmt) => self.execute_if_stmt(stmt),
            Stmt::While(stmt) => self.execute_while_stmt(stmt),
            Stmt::Function(stmt) => self.execute_fn_stmt(stmt),
            Stmt::Return(stmt) => self.execute_return_stmt(stmt),
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object> {
        trace!(?expr, "Evaluating expression");
        match expr {
            Expr::Binary(expr) => self.eval_binary(expr),
            Expr::Logical(expr) => self.eval_logical(expr),
            Expr::Grouping(expr) => self.eval_grouping(expr),
            Expr::Literal(expr) => self.eval_literal(expr),
            Expr::Unary(expr) => self.eval_unary(expr),
            Expr::Variable(_var) => self.eval_variable(expr),
            Expr::Assign(_assign) => self.eval_assign(expr),
            Expr::Call(expr) => self.eval_call(expr),
        }
    }

    pub fn execute_block(&mut self, statements: &Vec<Stmt>, environment: Environment) -> Result<()> {
        // TODO: consider passing environment to the visit methods instead
        self.enter_scope(environment);
        for statement in statements {
            self.execute(statement).inspect_err(|_| self.exit_scope().unwrap())?
        }
        self.exit_scope().unwrap();
        Ok(())
    }

    fn enter_scope(&mut self, nested_env: Environment) {
        trace!(values=?nested_env.values, "entering scope");
        let current_env = std::mem::replace(&mut self.environment, Box::new(nested_env));
        self.environment.enclosing = Some(current_env);
        trace!(top=?self.environment.values, enclosing=?self.environment.enclosing.as_ref().unwrap().values, "scope is entered");
    }

    fn exit_scope(&mut self) -> Result<()> {
        trace!("exiting scope");
        if let Some(enclosing) = self.environment.enclosing.take() {
            self.environment = enclosing;
            Ok(())
        } else {
            Err(LoxError::Internal {
                message: "Interpreter did not have an enclosing environment when exiting scope.".to_string(),
            })
        }
    }

    // TODO: shouldn't need to be mut
    fn execute_print_stmt(&mut self, stmt: &stmt::Print) -> Result<()> {
        let val = self.evaluate(&stmt.expression)?;
        if val < Object::Number(OrderedFloat(-1_f64)) {
            panic!()
        }
        println!("{}", val);
        Ok(())
    }

    fn execute_var_stmt(&mut self, stmt: &stmt::Var) -> Result<()> {
        let value = match &stmt.initializer {
            Some(init) => self.evaluate(init)?,
            None => Object::Null,
        };

        trace!(name = stmt.name.lexeme, ?value, "Defining in env");
        self.environment.define(stmt.name.lexeme.clone(), value);
        Ok(())
    }

    fn execute_if_stmt(&mut self, stmt: &stmt::If) -> Result<()> {
        let res = self.evaluate(&stmt.condition)?;
        if res.is_truthy() {
            self.execute(&stmt.then_branch)?;
        } else if let Some(ref eb) = stmt.else_branch {
            self.execute(eb)?;
        }

        Ok(())
    }

    fn execute_while_stmt(&mut self, stmt: &stmt::While) -> Result<()> {
        let mut res = self.evaluate(&stmt.condition)?;
        while res.is_truthy() {
            self.execute(&stmt.body)?;
            res = self.evaluate(&stmt.condition)?;
        }

        Ok(())
    }

    fn execute_fn_stmt(&mut self, stmt: &stmt::Function) -> Result<()> {
        let function = LoxFunction::new(Stmt::Function(stmt.clone()), *self.environment.clone()).map_err(|e| LoxError::Runtime {
            found: e.found,
            expected: e.expected,
            token: stmt.name.clone(),
        })?;
        self.environment
            .define(stmt.name.lexeme.clone(), Object::Callable(Rc::new(function)));
        Ok(())
    }

    fn execute_return_stmt(&mut self, stmt: &stmt::Return) -> Result<()> {
        let value = if let Some(ref val) = stmt.value {
            self.evaluate(val)?
        } else {
            Object::Null
        };
        Err(LoxError::Return { value })
    }

    fn eval_binary(&mut self, expr: &expr::Binary) -> Result<Object> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

        let obj = match expr.operator.typ {
            TokenType::Greater => Object::Boolean(left > right),
            TokenType::GreaterEqual => Object::Boolean(left >= right),
            TokenType::Less => Object::Boolean(left < right),
            TokenType::LessEqual => Object::Boolean(left <= right),
            TokenType::Minus => (left - right).map_err(|e| e.into_lox(&expr.operator))?,
            TokenType::Plus => (left + right).map_err(|e| e.into_lox(&expr.operator))?,
            TokenType::Slash => (left / right).map_err(|e| e.into_lox(&expr.operator))?,
            TokenType::Star => (left * right).map_err(|e| e.into_lox(&expr.operator))?,
            TokenType::EqualEqual => Object::Boolean(left == right),
            TokenType::BangEqual => Object::Boolean(left != right),
            _ => Object::Null,
        };

        Ok(obj)
    }

    fn eval_logical(&mut self, expr: &expr::Logical) -> Result<Object> {
        let left = self.evaluate(&expr.left)?;

        let truthy_left = left.is_truthy();
        match (&expr.operator.typ, truthy_left) {
            (&TokenType::Or, true) | (&TokenType::And, false) => Ok(left),
            _ => self.evaluate(&expr.right),
        }
    }

    fn eval_grouping(&mut self, expr: &expr::Grouping) -> Result<Object> {
        self.evaluate(&expr.expression)
    }

    fn eval_literal(&mut self, expr: &expr::Literal) -> Result<Object> {
        // TODO: get rid of clone
        Ok(expr.value.clone())
    }

    fn eval_unary(&mut self, expr: &expr::Unary) -> Result<Object> {
        let right = self.evaluate(&expr.right)?;
        let obj = match expr.operator.typ {
            TokenType::Minus => {
                let n = right.into_number().map_err(|e| e.into_lox(&expr.operator))?;
                Object::Number(OrderedFloat(-n))
            }
            TokenType::Bang => Object::Boolean(!right.is_truthy()),
            _ => {
                let token = expr.operator.clone(); // TODO: clone
                Err(LoxError::Runtime {
                    expected: "'!' or '-' unary operator".to_string(),
                    found: token.to_string(),
                    token,
                })
            }?,
        };

        Ok(obj)
    }

    fn eval_variable(&mut self, expr: &Expr) -> Result<Object> {
        self.lookup_variable(expr)
    }

    fn eval_assign(&mut self, expr: &Expr) -> Result<Object> {
        if let Expr::Assign(assign) = expr {
            let name = &assign.name;
            let value = self.evaluate(&assign.value)?;
            let distance = self.locals.get(expr);
            if let Some(distance) = distance {
                trace!(distance, ?value, ?name, "Assigning to local");
                self.environment.assign_at(distance, name, value.clone())?;
            } else {
                trace!(?value, ?name, "Assigning to global");
                self.globals.assign(name, value.clone())?;
            }
            Ok(value)
        } else {
            Err(LoxError::Internal {
                message: format!("Tried to assign with expr type {expr:?}"),
            })
        }
    }

    fn eval_call(&mut self, expr: &expr::Call) -> Result<Object> {
        let callee = self.evaluate(&expr.callee)?;
        let mut arguments = Vec::new();
        for argument in expr.arguments.iter() {
            arguments.push(self.evaluate(argument)?);
        }
        let function = callee;
        if arguments.len() as u8 != function.arity() {
            return Err(LoxError::Runtime {
                token: expr.paren.clone(),
                expected: format!("{} arguments", function.arity()),
                found: format!("{} arguments", arguments.len()),
            });
        }
        function.call(self, arguments).map_err(|e| e.into_lox(&expr.paren))
    }

    fn resolve(&mut self, expr: &Expr, i: u8) {
        println!("Locals before insert {:?}", self.locals);
        if self.locals.contains_key(expr) {
            panic!("Tried to insert {expr:?} at depth {i} over {:?}", self.locals.get(expr).unwrap());
        }
        self.locals.insert(expr.clone(), i);
        trace!(depth = i, ?expr, locals=?self.locals, "Inserted local");
    }

    fn lookup_variable(&mut self, expr: &Expr) -> Result<Object> {
        trace!(locals=?self.locals, "looking up {expr:?}");
        match expr {
            Expr::Variable(var) => {
                if let Some(distance) = self.locals.get(expr) {
                    let val = self.environment.get_at(distance, &var.name.lexeme);
                    trace!("var: found value {val:?} at distance {distance}\n{:?}", self.locals);
                    val
                } else {
                    trace!(globals=?self.globals.values, "var: no distance");
                    self.globals.get(&var.name)
                }
            }
            _ => panic!("Tried to lookup from invalid Expr type"),
        }
    }
}
