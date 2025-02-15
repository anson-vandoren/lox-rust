use std::{cmp, ops};

use snafu::Snafu;

use crate::{LoxError, interpreter::Interpreter, lox_callable::LoxCallable, token::Token};

#[derive(Clone, Debug)]
pub enum Object {
    String(String),
    Null,
    Number(f64),
    Boolean(bool),
}

//impl LoxCallable for Object {
//    fn call(&self, interpreter: Interpreter, arguments: Vec<Object>) -> Result<Object> {
//        match self {
//            _ => Err(ObjectRuntimeError {
//                found: format!("{:?}", self),
//                expected: "A function or class".to_string(),
//            }),
//        }
//    }
//
//    fn arity(&self) -> u8 {
//        todo!()
//    }
//}

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
        match self {
            Object::String(s) => other.as_string().map(|o| o == s).unwrap_or(false),
            Object::Null => matches!(other, Object::Null),
            Object::Number(s) => other.as_number().map(|o| o == s).unwrap_or(false),
            Object::Boolean(s) => other.as_bool().map(|o| o == s).unwrap_or(false),
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

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", s),
            Self::Null => write!(f, "nil"),
            Self::Number(n) => {
                if n.fract() == 0.0 {
                    // Don't print decimal places for integers
                    write!(f, "{}", n.trunc())
                } else {
                    write!(f, "{}", n)
                }
            }
            Self::Boolean(b) => write!(f, "{}", b),
        }
    }
}
