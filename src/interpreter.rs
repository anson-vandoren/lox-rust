use crate::{
    environment::Environment,
    expr::{self, Expr},
    object::Object,
    stmt::{self, Stmt},
    token_type::TokenType,
    LoxError, Result,
};

pub struct Interpreter {
    environment: Box<Environment>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Self {
            environment: Box::new(Environment::new()),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<()> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: Stmt) -> Result<()> {
        stmt.accept(self)
    }

    fn execute_ref(&mut self, stmt: &Stmt) -> Result<()> {
        stmt.accept_borrowed(self)
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

    fn execute_block(&mut self, statements: &Vec<Stmt>) {
        // TODO: consider passing environment to the visit methods instead
        self.enter_scope();
        for statement in statements {
            if let Err(e) = self.execute_ref(statement) {
                eprintln!("Failed to execute block, bailing early: {e}");
                break;
            }
        }
        self.exit_scope().unwrap();
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
                message: "Interpreter did not have an enclosing environment when exiting scope."
                    .to_string(),
            })
        }
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
                let n = right
                    .into_number()
                    .map_err(|e| e.into_lox(&expr.operator))?;
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

    fn visit_logical(&mut self, expr: expr::Logical) -> Result<Object> {
        let left = self.evaluate(*expr.left)?;

        let truthy_left = self.is_truthy(&left);
        match (expr.operator.typ, truthy_left) {
            (TokenType::Or, true) | (TokenType::And, false) => Ok(left),
            _ => self.evaluate(*expr.right),
        }
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
                let n = right
                    .into_number()
                    .map_err(|e| e.into_lox(&expr.operator))?;
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

    fn borrow_logical(&mut self, expr: &expr::Logical) -> Result<Object> {
        let left = self.evaluate_ref(&expr.left)?;

        let truthy_left = self.is_truthy(&left);
        match (&expr.operator.typ, truthy_left) {
            (&TokenType::Or, true) | (&TokenType::And, false) => Ok(left),
            _ => self.evaluate_ref(&expr.right),
        }
    }
}

impl stmt::Visitor<Result<()>> for &mut Interpreter {
    fn visit_block_stmt(&mut self, stmt: stmt::Block) -> Result<()> {
        self.execute_block(&stmt.statements);
        Ok(())
    }
    fn visit_expression_stmt(&mut self, stmt: stmt::Expression) -> Result<()> {
        self.evaluate(stmt.expression)?;
        Ok(())
    }

    fn visit_print_stmt(&mut self, stmt: stmt::Print) -> Result<()> {
        let value = self.evaluate(stmt.expression)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_var_stmt(&mut self, stmt: stmt::Var) -> Result<()> {
        let value = match stmt.initializer {
            Some(init) => self.evaluate(init)?,
            None => Object::Null,
        };

        self.environment.define(stmt.name.lexeme, value);
        Ok(())
    }

    fn visit_if_stmt(&mut self, stmt: stmt::If) -> Result<()> {
        let res = self.evaluate(stmt.condition)?;
        if self.is_truthy(&res) {
            self.execute(*stmt.then_branch)?;
        } else if let Some(eb) = stmt.else_branch {
            self.execute(*eb)?;
        }

        Ok(())
    }

    fn visit_while_stmt(&mut self, stmt: stmt::While) -> Result<()> {
        let mut res = self.evaluate_ref(&stmt.condition)?;
        while self.is_truthy(&res) {
            self.execute_ref(&stmt.body)?;
            res = self.evaluate_ref(&stmt.condition)?;
        }

        Ok(())
    }
}

impl stmt::BorrowingVisitor<Result<()>> for &mut Interpreter {
    fn borrow_block_stmt(&mut self, stmt: &stmt::Block) -> Result<()> {
        self.execute_block(&stmt.statements);
        Ok(())
    }
    fn borrow_expression_stmt(&mut self, stmt: &stmt::Expression) -> Result<()> {
        self.evaluate_ref(&stmt.expression)?;
        Ok(())
    }

    fn borrow_print_stmt(&mut self, stmt: &stmt::Print) -> Result<()> {
        let value = self.evaluate_ref(&stmt.expression)?;
        println!("{}", value);
        Ok(())
    }

    fn borrow_var_stmt(&mut self, stmt: &stmt::Var) -> Result<()> {
        let value = match stmt.initializer {
            Some(ref init) => self.evaluate_ref(init)?,
            None => Object::Null,
        };

        self.environment.define(stmt.name.lexeme.clone(), value);
        Ok(())
    }

    fn borrow_if_stmt(&mut self, stmt: &stmt::If) -> Result<()> {
        let res = self.evaluate_ref(&stmt.condition)?;
        if self.is_truthy(&res) {
            self.execute_ref(&stmt.then_branch)?;
        } else if let Some(ref eb) = stmt.else_branch {
            self.execute_ref(eb)?;
        }

        Ok(())
    }

    fn borrow_while_stmt(&mut self, stmt: &stmt::While) -> Result<()> {
        let mut res = self.evaluate_ref(&stmt.condition)?;
        while self.is_truthy(&res) {
            self.execute_ref(&stmt.body)?;
            res = self.evaluate_ref(&stmt.condition)?;
        }

        Ok(())
    }
}
