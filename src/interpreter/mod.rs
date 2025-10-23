pub mod environment;
pub mod resolver;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use environment::{Environment, RcCell};
use snafu::whatever;
use tracing::{instrument, trace, warn};

use super::{LoxError, Result};
use crate::{
    expr::{self, Expr},
    lox_callable::LoxCallable as _,
    lox_class::LoxClass,
    lox_function::LoxFunction,
    native::{assert_eq::LoxAssertEq, clock::LoxClock},
    object::{Literal, Object},
    stmt::{self, Stmt},
    token::Token,
    token_type::TokenType,
};

pub struct Interpreter {
    environment: RcCell<Environment>,
    pub globals: RcCell<Environment>,
    locals: HashMap<Token, u8>,
}

impl Default for Interpreter {
    fn default() -> Self {
        let bare = Rc::new(RefCell::new(Environment::new()));
        Self {
            environment: bare.clone(),
            globals: bare,
            locals: HashMap::new(),
        }
    }
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut globals = Environment::new();
        globals.define("clock".to_string(), Object::Callable(Rc::new(LoxClock {})));
        globals.define("assert_eq".to_string(), Object::Callable(Rc::new(LoxAssertEq {})));
        let globals = Rc::new(RefCell::new(globals));
        Self {
            environment: globals.clone(),
            globals,
            locals: HashMap::new(),
        }
    }

    #[instrument(skip(self, statements))]
    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<()> {
        for statement in statements {
            self.execute(&statement)?;
        }
        Ok(())
    }

    #[instrument(skip(self))]
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
            Stmt::Class(stmt) => self.execute_class_stmt(stmt),
        }
    }

    #[instrument(skip(self))]
    fn evaluate(&mut self, expr: &Expr) -> Result<Object> {
        trace!(?expr, "Evaluating expression");
        match expr {
            Expr::Binary(expr) => self.eval_binary(expr),
            Expr::Logical(expr) => self.eval_logical(expr),
            Expr::Grouping(expr) => self.eval_grouping(expr),
            Expr::Literal(expr) => self.eval_literal(expr),
            Expr::Unary(expr) => self.eval_unary(expr),
            Expr::Variable(var) => self.eval_variable(var),
            Expr::Assign(assign) => self.eval_assign(assign),
            Expr::Call(expr) => self.eval_call(expr),
            Expr::Get(expr) => self.eval_get(expr),
            Expr::Set(expr) => self.eval_set(expr),
            Expr::This(expr) => self.eval_this(expr),
        }
    }

    #[instrument(skip(self))]
    fn evaluate_literal(&mut self, expr: &Expr) -> Result<Literal> {
        trace!(?expr, "Evaluating literal expression");
        let as_obj = self.evaluate(expr)?;
        if let Object::Literal(lit) = as_obj {
            Ok(lit)
        } else {
            Err(LoxError::Internal {
                message: format!("Expected a literal, found {:?}", as_obj),
            })
        }
    }

    #[instrument(skip(self), err)]
    pub fn execute_block(&mut self, statements: &Vec<Stmt>, environment: Environment) -> Result<()> {
        trace!(?environment, ?statements, ">>execute_block()");
        // TODO: consider passing environment to the visit methods instead
        //
        let environment = Rc::new(RefCell::new(environment));
        let original_env = std::mem::replace(&mut self.environment, environment);

        let result = (|| {
            for statement in statements {
                self.execute(statement).inspect_err(|err| {
                    warn!(?err, ?statement, "Failed to execute statement in block");
                })?
            }
            Ok(())
        })();
        self.environment = original_env;
        result
    }

    // TODO: shouldn't need to be mut
    fn execute_print_stmt(&mut self, stmt: &stmt::Print) -> Result<()> {
        let val = self.evaluate(&stmt.expression)?;
        println!("{}", val);
        Ok(())
    }

    fn execute_var_stmt(&mut self, stmt: &stmt::Var) -> Result<()> {
        let value = match &stmt.initializer {
            Some(init) => self.evaluate(init)?,
            None => Object::Literal(Literal::Null),
        };

        trace!(name = stmt.name.lexeme, ?value, "Defining in env");
        self.environment.borrow_mut().define(stmt.name.lexeme.clone(), value);
        Ok(())
    }

    fn execute_if_stmt(&mut self, stmt: &stmt::If) -> Result<()> {
        let res = self.evaluate_literal(&stmt.condition)?;
        if res.is_truthy() {
            self.execute(&stmt.then_branch)?;
        } else if let Some(ref eb) = stmt.else_branch {
            self.execute(eb)?;
        }

        Ok(())
    }

    fn execute_while_stmt(&mut self, stmt: &stmt::While) -> Result<()> {
        let mut res = self.evaluate_literal(&stmt.condition)?;
        while res.is_truthy() {
            self.execute(&stmt.body)?;
            res = self.evaluate_literal(&stmt.condition)?;
        }

        Ok(())
    }

    fn execute_fn_stmt(&mut self, stmt: &stmt::Function) -> Result<()> {
        let function = LoxFunction::new(stmt.clone(), self.environment.clone());
        self.environment
            .borrow_mut()
            .define(stmt.name.lexeme.clone(), Object::Callable(Rc::new(function)));
        Ok(())
    }

    fn execute_return_stmt(&mut self, stmt: &stmt::Return) -> Result<()> {
        let value = if let Some(ref val) = stmt.value {
            self.evaluate(val)?
        } else {
            Object::Literal(Literal::Null)
        };
        // TODO: why not regular return here?
        Err(LoxError::Return { value })
    }

    fn execute_class_stmt(&mut self, stmt: &stmt::Class) -> Result<()> {
        self.environment
            .borrow_mut()
            .define(stmt.name.lexeme.clone(), Object::Literal(Literal::Null));

        let mut methods = HashMap::new();
        for method in stmt.methods.iter() {
            let function = LoxFunction::new(method.clone(), self.environment.clone());
            methods.insert(method.name.lexeme.clone(), function);
        }

        let class = LoxClass::new(&stmt.name.lexeme, methods);
        self.environment.borrow_mut().assign(&stmt.name, Object::Callable(Rc::new(class)))?;
        Ok(())
    }

    fn eval_binary(&mut self, expr: &expr::Binary) -> Result<Object> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

        let obj = match expr.operator.typ {
            TokenType::Greater => (left > right).into(),
            TokenType::GreaterEqual => (left >= right).into(),
            TokenType::Less => (left < right).into(),
            TokenType::LessEqual => (left <= right).into(),
            TokenType::Minus => (left - right).map_err(|e| e.add_line(expr.operator.line))?,
            TokenType::Plus => (left + right).map_err(|e| e.add_line(expr.operator.line))?,
            TokenType::Slash => (left / right).map_err(|e| e.add_line(expr.operator.line))?,
            TokenType::Star => (left * right).map_err(|e| e.add_line(expr.operator.line))?,
            TokenType::EqualEqual => (left == right).into(),
            TokenType::BangEqual => (left != right).into(),
            _ => Object::Literal(Literal::Null),
        };

        Ok(obj)
    }

    fn eval_logical(&mut self, expr: &expr::Logical) -> Result<Object> {
        let left = self.evaluate_literal(&expr.left)?;

        let truthy_left = left.is_truthy();
        match (&expr.operator.typ, truthy_left) {
            (&TokenType::Or, true) | (&TokenType::And, false) => Ok(Object::Literal(left)),
            _ => self.evaluate(&expr.right),
        }
    }

    fn eval_grouping(&mut self, expr: &expr::Grouping) -> Result<Object> {
        self.evaluate(&expr.expression)
    }

    fn eval_literal(&mut self, expr: &expr::Literal) -> Result<Object> {
        // TODO: get rid of clone
        Ok(Object::Literal(expr.value.clone()))
    }

    fn eval_unary(&mut self, expr: &expr::Unary) -> Result<Object> {
        let right = self.evaluate_literal(&expr.right)?;
        let obj = match expr.operator.typ {
            TokenType::Minus => {
                let n = right.into_number().map_err(|e| e.add_line(expr.operator.line))?;
                Object::from(-n)
            }
            TokenType::Bang => (!right.is_truthy()).into(),
            _ => {
                let token = expr.operator.clone(); // TODO: clone
                Err(LoxError::Runtime {
                    expected: "'!' or '-' unary operator".to_string(),
                    found: token.to_string(),
                    line: Some(token.line),
                })
            }?,
        };

        Ok(obj)
    }

    fn eval_variable(&mut self, expr: &expr::Variable) -> Result<Object> {
        self.lookup_variable(expr)
    }

    fn eval_assign(&mut self, assign: &expr::Assign) -> Result<Object> {
        let name = &assign.name;
        let value = self.evaluate(&assign.value)?;
        let distance = self.locals.get(&assign.name);
        if let Some(distance) = distance {
            trace!(distance, ?value, ?name, "Assigning to local");
            self.environment.borrow_mut().assign_at(distance, &name.lexeme, value.clone())?;
        } else {
            trace!(?value, ?name, "Assigning to global");
            self.environment.borrow_mut().assign(name, value.clone())?;
        }
        Ok(value)
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
                line: Some(expr.paren.line),
                expected: format!("{} arguments", function.arity()),
                found: format!("{} arguments", arguments.len()),
            });
        }
        function.call(self, arguments).map_err(|e| e.add_line(expr.paren.line))
    }

    fn resolve(&mut self, token: &Token, i: u8) {
        if self.locals.contains_key(token) {
            panic!("Tried to insert {token:?} at depth {i} over {:?}", self.locals.get(token).unwrap());
        }
        self.locals.insert(token.clone(), i);
        trace!(depth = i, ?token, locals=?self.locals, "Inserted local");
    }

    fn lookup_variable(&mut self, var: &expr::Variable) -> Result<Object> {
        trace!(locals=?self.locals, "looking up {var:?}");
        if let Some(distance) = self.locals.get(&var.name) {
            let val = self.environment.borrow_mut().get_at(distance, &var.name.lexeme);
            trace!("var: found value {val:?} at distance {distance}\n{:?}", self.locals);
            val
        } else {
            trace!(globals=?self.globals.borrow().values, "var: no distance");
            self.environment.borrow().get(&var.name)
        }
    }

    fn eval_get(&mut self, expr: &expr::Get) -> Result<Object> {
        let object = self.evaluate(&expr.object)?;
        if let Object::Instance(instance) = object {
            return instance.get(&expr.name);
        }

        Err(LoxError::Internal {
            message: "Only instances have properties.".to_string(),
        })
    }

    fn eval_set(&mut self, expr: &expr::Set) -> Result<Object> {
        let object = self.evaluate(&expr.object)?;

        if let Object::Instance(mut object) = object {
            let value = self.evaluate(&expr.value)?;
            object.set(expr.name.clone(), value.clone());
            trace!(?expr,?object,  ?value, env = ?self.environment.borrow().values, "Object after setting");

            // Update env with the mutated object
            //let Expr::Variable(ref var_expr) = *expr.object else {
            //    whatever!("wrong kind of expr: {:?}", *expr.object)
            //};
            let t = match *expr.object {
                Expr::Variable(ref var_expr) => var_expr.name.clone(),
                Expr::This(ref t) => t.keyword.clone(),
                _ => whatever!("Wrong kind of expr"),
            };
            trace!(name = ?t, "Updating mutated obj");
            self.environment.borrow_mut().assign(&t, Object::Instance(object))?;

            Ok(value)
        } else {
            Err(LoxError::Runtime {
                found: format!("{:?}", object),
                expected: "A LoxInstance".into(),
                line: Some(expr.name.line),
            })
        }
    }

    fn eval_this(&mut self, expr: &expr::This) -> Result<Object> {
        trace!(?expr, ">>eval_this()");
        let var = expr::Variable {
            name: expr.keyword.clone(),
        };
        let var = self.lookup_variable(&var);
        trace!(?var, "<<eval_this()");
        var
    }
}
