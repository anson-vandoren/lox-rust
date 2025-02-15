use tracing::instrument;

use crate::{
    LoxError, Result,
    environment::Environment,
    expr::{self, Expr},
    object::Object,
    stmt::{self, Stmt},
    token_type::TokenType,
};

pub struct Interpreter {
    environment: Box<Environment>,
    globals: Environment,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Environment::new();
        // globals.define("clock".to_string(), LoxClock {});
        Self {
            environment: Box::new(globals.clone()),
            globals,
        }
    }

    #[instrument(skip(self, statements), err, ret, level = "trace")]
    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Print(stmt) => self.execute_print_stmt(stmt),
            Stmt::Block(stmt) => self.execute_block(&stmt.statements),
            Stmt::Expression(stmt) => self.evaluate(&stmt.expression).map(|_| ()),
            Stmt::Var(stmt) => self.execute_var_stmt(stmt),
            Stmt::If(stmt) => self.execute_if_stmt(stmt),
            Stmt::While(stmt) => self.execute_while_stmt(stmt),
            Stmt::Function(function) => todo!(),
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object> {
        match expr {
            Expr::Binary(expr) => self.eval_binary(expr),
            Expr::Logical(expr) => self.eval_logical(expr),
            Expr::Grouping(expr) => self.eval_grouping(expr),
            Expr::Literal(expr) => self.eval_literal(expr),
            Expr::Unary(expr) => self.eval_unary(expr),
            Expr::Variable(expr) => self.eval_variable(expr),
            Expr::Assign(expr) => self.eval_assign(expr), // TODO: this is what makes it mut
            Expr::Call(expr) => self.eval_call(expr),
        }
    }

    fn is_truthy(&self, obj: &Object) -> bool {
        match obj {
            Object::Null => false,
            Object::Boolean(b) => *b,
            _ => true,
        }
    }

    fn execute_block(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        // TODO: consider passing environment to the visit methods instead
        self.enter_scope();
        for statement in statements {
            self.execute(statement).inspect_err(|_| self.exit_scope().unwrap())?
        }
        self.exit_scope().unwrap();
        Ok(())
    }

    fn enter_scope(&mut self) {
        let current_env = std::mem::replace(&mut self.environment, Box::new(Environment::new()));
        self.environment = Box::new(Environment::with_enclosing(current_env));
    }

    fn exit_scope(&mut self) -> Result<()> {
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
        println!("{}", self.evaluate(&stmt.expression)?);
        Ok(())
    }

    fn execute_var_stmt(&mut self, stmt: &stmt::Var) -> Result<()> {
        let value = match &stmt.initializer {
            Some(init) => self.evaluate(init)?,
            None => Object::Null,
        };

        self.environment.define(stmt.name.lexeme.clone(), value);
        Ok(())
    }

    fn execute_if_stmt(&mut self, stmt: &stmt::If) -> Result<()> {
        let res = self.evaluate(&stmt.condition)?;
        if self.is_truthy(&res) {
            self.execute(&stmt.then_branch)?;
        } else if let Some(ref eb) = stmt.else_branch {
            self.execute(eb)?;
        }

        Ok(())
    }

    fn execute_while_stmt(&mut self, stmt: &stmt::While) -> Result<()> {
        let mut res = self.evaluate(&stmt.condition)?;
        while self.is_truthy(&res) {
            self.execute(&stmt.body)?;
            res = self.evaluate(&stmt.condition)?;
        }

        Ok(())
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

        let truthy_left = self.is_truthy(&left);
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
                Object::Number(-n)
            }
            TokenType::Bang => Object::Boolean(!self.is_truthy(&right)),
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

    fn eval_variable(&mut self, expr: &expr::Variable) -> Result<Object> {
        self.environment.get(&expr.name)
    }

    fn eval_assign(&mut self, expr: &expr::Assign) -> Result<Object> {
        let value = self.evaluate(&expr.value)?;
        self.environment.assign(&expr.name, value.clone())?; // TODO: clone
        Ok(value)
    }

    fn eval_call(&mut self, expr: &expr::Call) -> Result<Object> {
        // let callee = self.evaluate(*expr.callee)?;
        //
        // let arguments = Vec::new();
        // for argument in expr.arguments {
        //    arguments.push(self.evaluate(argument)?);
        //}
        // let function = callee;
        // if arguments.len() != function.arity() {
        //    return Err(LoxError::Runtime {
        //        token: expr.paren,
        //        expected: format!("{} arguments", function.arity()),
        //        found: format!("{} arguments", arguments.len()),
        //    });
        //}
        // function.call(self, arguments)
        todo!()
    }
}
