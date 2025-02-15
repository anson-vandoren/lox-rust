use tracing::instrument;

use crate::{
    environment::Environment,
    expr::{self, Expr},
    native::clock::LoxClock,
    object::Object,
    stmt::{self, Stmt},
    token_type::TokenType,
    LoxError, Result,
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
            Stmt::Expression(stmt) => self.evaluate_ref(&stmt.expression).map(|_| ()),
            Stmt::Var(stmt) => self.execute_var_stmt(stmt),
            Stmt::If(stmt) => self.execute_if_stmt(stmt),
            Stmt::While(stmt) => self.execute_while_stmt(stmt),
            Stmt::Function(function) => todo!(),
        }
    }

    fn evaluate(&mut self, expr: Expr) -> Result<Object> {
        expr.accept::<Result<Object>>(self)
    }

    fn evaluate_ref(&mut self, expr: &Expr) -> Result<Object> {
        expr.accept_borrowed::<Result<Object>>(self)
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
        println!("{}", self.evaluate_ref(&stmt.expression)?);
        Ok(())
    }

    fn execute_var_stmt(&mut self, stmt: &stmt::Var) -> Result<()> {
        let value = match &stmt.initializer {
            Some(init) => self.evaluate_ref(init)?,
            None => Object::Null,
        };

        self.environment.define(stmt.name.lexeme.clone(), value);
        Ok(())
    }

    fn execute_if_stmt(&mut self, stmt: &stmt::If) -> Result<()> {
        let res = self.evaluate_ref(&stmt.condition)?;
        if self.is_truthy(&res) {
            self.execute(&stmt.then_branch)?;
        } else if let Some(ref eb) = stmt.else_branch {
            self.execute(eb)?;
        }

        Ok(())
    }

    fn execute_while_stmt(&mut self, stmt: &stmt::While) -> Result<()> {
        let mut res = self.evaluate_ref(&stmt.condition)?;
        while self.is_truthy(&res) {
            self.execute(&stmt.body)?;
            res = self.evaluate_ref(&stmt.condition)?;
        }

        Ok(())
    }
}

impl expr::Visitor<Result<Object>> for &mut Interpreter {
    fn visit_binary(&mut self, expr: crate::expr::Binary) -> Result<Object> {
        let left = self.evaluate(*expr.left)?;
        let right = self.evaluate(*expr.right)?;

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

    fn visit_logical(&mut self, expr: expr::Logical) -> Result<Object> {
        let left = self.evaluate(*expr.left)?;

        let truthy_left = self.is_truthy(&left);
        match (expr.operator.typ, truthy_left) {
            (TokenType::Or, true) | (TokenType::And, false) => Ok(left),
            _ => self.evaluate(*expr.right),
        }
    }

    fn visit_grouping(&mut self, expr: expr::Grouping) -> Result<Object> {
        self.evaluate(*expr.expression)
    }

    fn visit_literal(&self, expr: expr::Literal) -> Result<Object> {
        Ok(expr.value)
    }

    fn visit_unary(&mut self, expr: expr::Unary) -> Result<Object> {
        let right = self.evaluate(*expr.right)?;
        let obj = match expr.operator.typ {
            TokenType::Minus => {
                let n = right.into_number().map_err(|e| e.into_lox(&expr.operator))?;
                Object::Number(-n)
            }
            TokenType::Bang => Object::Boolean(!self.is_truthy(&right)),
            _ => Err(LoxError::Runtime {
                expected: "'!' or '-' unary operator".to_string(),
                found: expr.operator.to_string(),
                token: expr.operator,
            })?,
        };

        Ok(obj)
    }

    fn visit_variable(&self, expr: expr::Variable) -> Result<Object> {
        self.environment.get(expr.name)
    }

    fn visit_assign(&mut self, expr: expr::Assign) -> Result<Object> {
        let value = self.evaluate(*expr.value)?;
        self.environment.assign(&expr.name, value.clone())?;
        Ok(value)
    }

    fn visit_call(&mut self, expr: expr::Call) -> Result<Object> {
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

impl expr::BorrowingVisitor<Result<Object>> for &mut Interpreter {
    fn borrow_binary(&mut self, expr: &crate::expr::Binary) -> Result<Object> {
        let left = self.evaluate_ref(&expr.left)?;
        let right = self.evaluate_ref(&expr.right)?;

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

    fn borrow_logical(&mut self, expr: &expr::Logical) -> Result<Object> {
        let left = self.evaluate_ref(&expr.left)?;

        let truthy_left = self.is_truthy(&left);
        match (&expr.operator.typ, truthy_left) {
            (&TokenType::Or, true) | (&TokenType::And, false) => Ok(left),
            _ => self.evaluate_ref(&expr.right),
        }
    }

    fn borrow_grouping(&mut self, expr: &expr::Grouping) -> Result<Object> {
        self.evaluate_ref(&expr.expression)
    }

    fn borrow_literal(&mut self, expr: &expr::Literal) -> Result<Object> {
        Ok(expr.value.clone())
    }

    fn borrow_unary(&mut self, expr: &expr::Unary) -> Result<Object> {
        let right = self.evaluate_ref(&expr.right)?;
        let obj = match expr.operator.typ {
            TokenType::Minus => {
                let n = right.into_number().map_err(|e| e.into_lox(&expr.operator))?;
                Object::Number(-n)
            }
            TokenType::Bang => Object::Boolean(!self.is_truthy(&right)),
            _ => {
                let token = expr.operator.clone();
                Err(LoxError::Runtime {
                    expected: "'!' or '-' unary operator".to_string(),
                    found: token.to_string(),
                    token,
                })
            }?,
        };

        Ok(obj)
    }

    fn borrow_variable(&mut self, expr: &expr::Variable) -> Result<Object> {
        self.environment.get(expr.name.clone())
    }

    fn borrow_assign(&mut self, expr: &expr::Assign) -> Result<Object> {
        let value = self.evaluate_ref(&expr.value)?;
        self.environment.assign(&expr.name, value.clone())?;
        Ok(value)
    }

    fn borrow_call(&mut self, expr: &expr::Call) -> Result<Object> {
        todo!()
    }
}
