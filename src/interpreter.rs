use crate::{
    expr::{Expr, Visitor},
    object::Object,
    token_type::TokenType,
    LoxError, Result,
};

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Interpreter {
        Self {}
    }

    pub fn interpret(&self, expr: Expr) -> Result<Object> {
        let value = self.evaluate(expr)?;
        println!("{}", value);
        Ok(value)
    }

    fn evaluate(&self, expr: Expr) -> Result<Object> {
        expr.accept::<Result<Object>>(self)
    }

    fn is_truthy(&self, obj: Object) -> bool {
        match obj {
            Object::Null => false,
            Object::Boolean(b) => b,
            _ => true,
        }
    }
}

impl Visitor<Result<Object>> for &Interpreter {
    fn visit_binary(&self, expr: crate::expr::Binary) -> Result<Object> {
        let left = self.evaluate(*expr.left)?;
        let right = self.evaluate(*expr.right)?;

        let obj = match expr.operator.typ {
            TokenType::Greater => Object::Boolean(left > right),
            TokenType::GreaterEqual => Object::Boolean(left >= right),
            TokenType::Less => Object::Boolean(left < right),
            TokenType::LessEqual => Object::Boolean(left <= right),
            TokenType::Minus => (left - right).map_err(|e| e.to_lox(expr.operator))?,
            TokenType::Plus => (left + right).map_err(|e| e.to_lox(expr.operator))?,
            TokenType::Slash => (left / right).map_err(|e| e.to_lox(expr.operator))?,
            TokenType::Star => (left * right).map_err(|e| e.to_lox(expr.operator))?,
            TokenType::EqualEqual => Object::Boolean(left == right),
            TokenType::BangEqual => Object::Boolean(left != right),
            _ => Object::Null,
        };

        Ok(obj)
    }

    fn visit_grouping(&self, expr: crate::expr::Grouping) -> Result<Object> {
        self.evaluate(*expr.expression)
    }

    fn visit_literal(&self, expr: crate::expr::Literal) -> Result<Object> {
        Ok(expr.value)
    }

    fn visit_unary(&self, expr: crate::expr::Unary) -> Result<Object> {
        let right = self.evaluate(*expr.right)?;
        let obj = match expr.operator.typ {
            TokenType::Minus => {
                let n = right.into_number().map_err(|e| e.to_lox(expr.operator))?;
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
}
