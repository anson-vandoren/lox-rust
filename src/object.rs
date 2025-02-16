use std::{cmp, fmt, ops, rc::Rc};

use snafu::Snafu;

use crate::{interpreter::Interpreter, lox_callable::LoxCallable, token::Token, LoxError};

#[derive(Clone)]
pub enum Object {
    String(String),
    Null,
    Number(f64),
    Boolean(bool),
    Callable(Rc<dyn LoxCallable>),
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Null => false,
            Object::Boolean(b) => *b,
            _ => true,
        }
    }
}

impl fmt::Debug for Object {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::String(s) => write!(f, "\"{}\"", &s),
            Object::Null => fmt::Formatter::write_str(f, "Null"),
            Object::Number(n) => write!(f, "{}", &n),
            Object::Boolean(b) => write!(f, "{}", &b),
            Object::Callable(c) => write!(f, "{c}"),
        }
    }
}

impl LoxCallable for Object {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object> {
        match self {
            Self::Callable(c) => c.call(interpreter, arguments),
            _ => panic!("{:?} is not a LoxCallable", &self),
        }
    }

    fn arity(&self) -> u8 {
        match self {
            Self::Callable(c) => c.arity(),
            _ => panic!("{:?} is not a LoxCallable", &self),
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Callable(c) => c.name(),
            _ => panic!("{:?} is not a LoxCallable", &self),
        }
    }
}

#[derive(Debug, Snafu)]
#[snafu(display("Object comparison error: expected {expected}, but found {found}"))]
pub struct ObjectRuntimeError {
    pub found: String,
    pub expected: String,
}

impl ObjectRuntimeError {
    pub fn into_lox(self, token: &Token) -> LoxError {
        LoxError::Runtime {
            found: self.found,
            expected: self.expected,
            token: token.clone(),
        }
    }
}

type Result<T> = std::result::Result<T, ObjectRuntimeError>;

impl Object {
    pub fn into_number(self) -> Result<f64> {
        match self {
            Object::Number(n) => Ok(n),
            _ => Err(ObjectRuntimeError {
                found: self.to_string(),
                expected: "f64".to_string(),
            }),
        }
    }

    fn as_number(&self) -> Result<&f64> {
        match self {
            Object::Number(n) => Ok(n),
            _ => Err(ObjectRuntimeError {
                found: self.to_string(),
                expected: "f64".to_string(),
            }),
        }
    }

    pub fn into_string(self) -> Result<String> {
        match self {
            Object::String(s) => Ok(s),
            _ => Err(ObjectRuntimeError {
                found: self.to_string(),
                expected: "string".to_string(),
            }),
        }
    }

    fn as_string(&self) -> Result<&str> {
        match self {
            Object::String(s) => Ok(s),
            _ => Err(ObjectRuntimeError {
                found: self.to_string(),
                expected: "string".to_string(),
            }),
        }
    }

    fn as_bool(&self) -> Result<&bool> {
        match self {
            Object::Boolean(b) => Ok(b),
            _ => Err(ObjectRuntimeError {
                found: self.to_string(),
                expected: "boolean".to_string(),
            }),
        }
    }
}

impl ops::Add for Object {
    type Output = Result<Object>;

    fn add(self, rhs: Self) -> Self::Output {
        if matches!(self, Object::Number(_)) && matches!(rhs, Object::Number(_)) {
            let lhs = self.into_number()?;
            let rhs = rhs.into_number()?;
            return Ok(Object::Number(lhs + rhs));
        }
        if matches!(self, Object::String(_)) && matches!(rhs, Object::String(_)) {
            let lhs = self.into_string()?;
            let rhs = rhs.into_string()?;
            return Ok(Object::String(format!("{}{}", lhs, rhs)));
        }

        Err(ObjectRuntimeError {
            found: format!("{} + {}", self, rhs),
            expected: "String + String, or Number + Number".to_string(),
        })
    }
}

impl ops::Sub for Object {
    type Output = Result<Object>;

    fn sub(self, rhs: Self) -> Self::Output {
        let lhs = self.into_number()?;
        let rhs = rhs.into_number()?;
        Ok(Object::Number(lhs - rhs))
    }
}

impl ops::Neg for Object {
    type Output = Result<Object>;

    fn neg(self) -> Self::Output {
        match self {
            Object::Number(n) => Ok(Object::Number(-n)),
            _ => Err(ObjectRuntimeError {
                found: self.to_string(),
                expected: "a number to negate".to_string(),
            }),
        }
    }
}

impl ops::Div for Object {
    type Output = Result<Object>;

    fn div(self, rhs: Self) -> Self::Output {
        let lhs = self.into_number()?;
        let rhs = rhs.into_number()?;
        Ok(Object::Number(lhs / rhs))
    }
}

impl ops::Mul for Object {
    type Output = Result<Object>;

    fn mul(self, rhs: Self) -> Self::Output {
        let lhs = self.into_number()?;
        let rhs = rhs.into_number()?;
        Ok(Object::Number(lhs * rhs))
    }
}

impl cmp::PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let us = self.as_number().ok()?;
        let them = other.as_number().ok()?;
        us.partial_cmp(them)
    }
}

impl cmp::PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::String(s1), Object::String(s2)) => s1 == s2,
            (Object::Null, Object::Null) => true,
            (Object::Number(n1), Object::Number(n2)) => n1 == n2,
            (Object::Boolean(b1), Object::Boolean(b2)) => b1 == b2,
            (Object::Callable(c1), Object::Callable(c2)) => c1.name() == c2.name() && c1.arity() == c2.arity(),
            _ => false,
        }
    }
}

impl From<String> for Object {
    fn from(value: String) -> Self {
        Object::String(value)
    }
}

impl From<f64> for Object {
    fn from(value: f64) -> Self {
        Object::Number(value)
    }
}

impl From<()> for Object {
    fn from(_value: ()) -> Self {
        Object::Null
    }
}

impl From<bool> for Object {
    fn from(value: bool) -> Self {
        Object::Boolean(value)
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::String(s) => write!(f, "{}", s),
            Object::Null => write!(f, "nil"),
            Object::Number(n) => {
                if n.fract() == 0.0 {
                    // Don't print decimal places for integers
                    write!(f, "{}", n.trunc())
                } else {
                    write!(f, "{}", n)
                }
            }
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Callable(c) => write!(f, "callable <{}>", c.name()),
        }
    }
}
