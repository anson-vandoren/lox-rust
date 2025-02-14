use crate::{
    environment::Environment,
    expr::{self, Expr},
    object::Object,
    stmt::{self, Stmt},
    token_type::TokenType,
    LoxError, Result,
};

pub struct Interpreter {
    environment: Option<Environment>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Self {
            environment: Some(Environment::new()),
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

    fn evaluate(&mut self, expr: Expr) -> Result<Object> {
        expr.accept::<Result<Object>>(self)
    }

    fn is_truthy(&self, obj: Object) -> bool {
        match obj {
            Object::Null => false,
            Object::Boolean(b) => b,
            _ => true,
        }
    }

    fn execute_block(&mut self, statements: Vec<Stmt>) {
        let env = self
            .environment
            .take()
            .expect("Intepreter had no Environment");
        self.environment = Some(env.new_enclosing());

        // restore our env
        self.environment = self
            .environment
            .take()
            .expect("Interpreter had no Environment")
            .into_outer();
        todo!()
    }
}

impl expr::Visitor<Result<Object>> for &mut Interpreter<'_> {
    fn visit_binary(&mut self, expr: crate::expr::Binary) -> Result<Object> {
        let left = self.evaluate(*expr.left)?;
        let right = self.evaluate(*expr.right)?;

        let obj = match expr.operator.typ {
            TokenType::Greater => Object::Boolean(left > right),
            TokenType::GreaterEqual => Object::Boolean(left >= right),
            TokenType::Less => Object::Boolean(left < right),
            TokenType::LessEqual => Object::Boolean(left <= right),
            TokenType::Minus => (left - right).map_err(|e| e.into_lox(expr.operator))?,
            TokenType::Plus => (left + right).map_err(|e| e.into_lox(expr.operator))?,
            TokenType::Slash => (left / right).map_err(|e| e.into_lox(expr.operator))?,
            TokenType::Star => (left * right).map_err(|e| e.into_lox(expr.operator))?,
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
                let n = right.into_number().map_err(|e| e.into_lox(expr.operator))?;
                Object::Number(-n)
            }
            TokenType::Bang => Object::Boolean(!self.is_truthy(right)),
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
        self.environment.assign(expr.name, value.clone())?;
        Ok(value)
    }
}

impl stmt::Visitor<Result<()>> for &mut Interpreter {
    fn visit_block_stmt(&mut self, stmt: stmt::Block) -> Result<()> {
        self.execute_block(stmt.statements);
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
}
