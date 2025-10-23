use std::{cmp, fmt, ops, rc::Rc};

use ordered_float::OrderedFloat;

use crate::{LoxError, interpreter::Interpreter, lox_callable::LoxCallable, lox_instance::LoxInstance};

#[derive(Clone)]
pub enum Object {
    Callable(Rc<dyn LoxCallable>),
    Instance(LoxInstance),
    Literal(Literal),
}

impl From<bool> for Object {
    fn from(value: bool) -> Self {
        Object::Literal(Literal::Boolean(value))
    }
}

impl From<f64> for Object {
    fn from(value: f64) -> Self {
        Object::Literal(Literal::Number(OrderedFloat(value)))
    }
}

impl From<OrderedFloat<f64>> for Object {
    fn from(value: OrderedFloat<f64>) -> Self {
        Object::Literal(Literal::Number(value))
    }
}

impl Eq for Object {}

impl fmt::Debug for Object {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Callable(c) => write!(f, "{c}"),
            Object::Instance(c) => write!(f, "{c}"),
            Object::Literal(literal) => write!(f, "{literal:?}"),
        }
    }
}

impl LoxCallable for Object {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object, LoxError> {
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

impl ops::Add for Object {
    type Output = Result<Object, LoxError>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Literal(first), Self::Literal(second)) => Ok(Object::Literal((first + second)?)),
            _ => Err(LoxError::Runtime {
                found: "non-literal operands".into(),
                expected: "String + String, or Number + Number".to_string(),
                line: None,
            }),
        }
    }
}

impl ops::Sub for Object {
    type Output = Result<Object, LoxError>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Literal(first), Self::Literal(second)) => Ok(Object::Literal((first - second)?)),
            _ => Err(LoxError::Runtime {
                found: "non-literal operands".into(),
                expected: "String + String, or Number + Number".to_string(),
                line: None,
            }),
        }
    }
}

impl ops::Neg for Object {
    type Output = Result<Object, LoxError>;

    fn neg(self) -> Self::Output {
        if let Self::Literal(s) = self {
            Ok(Object::Literal((-s)?))
        } else {
            Err(LoxError::Runtime {
                found: self.to_string(),
                expected: "a number to negate".to_string(),
                line: None,
            })
        }
    }
}

impl ops::Div for Object {
    type Output = Result<Object, LoxError>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Literal(first), Self::Literal(second)) => Ok(Object::Literal((first / second)?)),
            _ => Err(LoxError::Runtime {
                found: "non-literal operands".into(),
                expected: "String + String, or Number + Number".to_string(),
                line: None,
            }),
        }
    }
}

impl ops::Mul for Object {
    type Output = Result<Object, LoxError>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Literal(first), Self::Literal(second)) => Ok(Object::Literal((first * second)?)),
            _ => Err(LoxError::Runtime {
                found: "non-literal operands".into(),
                expected: "String + String, or Number + Number".to_string(),
                line: None,
            }),
        }
    }
}

impl cmp::PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (Self::Literal(first), Self::Literal(second)) => first.partial_cmp(second),
            _ => None,
        }
    }
}

impl cmp::PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Callable(c1), Object::Callable(c2)) => c1.name() == c2.name() && c1.arity() == c2.arity(),
            (Object::Literal(l1), Object::Literal(l2)) => l1 == l2,
            _ => false,
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Callable(c) => write!(f, "callable <{}>", c.name()),
            Object::Instance(c) => write!(f, "{}", c),
            Object::Literal(literal) => write!(f, "{literal}"),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Literal {
    String(String),
    Null,
    Number(OrderedFloat<f64>),
    Boolean(bool),
}

impl std::ops::Add for Literal {
    type Output = Result<Literal, LoxError>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Literal::Number(first), Literal::Number(second)) => Ok(Literal::Number(first + second)),
            (Literal::String(first), Literal::String(second)) => Ok(format!("{}{}", first, second).into()),
            _ => Err(LoxError::Runtime {
                found: "mismatched operands".into(),
                expected: "string + string, or number + number".into(),
                line: None,
            }),
        }
    }
}

impl std::ops::Sub for Literal {
    type Output = Result<Literal, LoxError>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Literal::Number(first), Literal::Number(second)) => Ok(Literal::Number(first - second)),
            _ => Err(LoxError::Runtime {
                found: "non-number operand(s)".into(),
                expected: "number + number".into(),
                line: None,
            }),
        }
    }
}

impl ops::Neg for Literal {
    type Output = Result<Literal, LoxError>;

    fn neg(self) -> Self::Output {
        match self {
            Literal::Number(n) => Ok(Literal::Number(-n)),
            _ => Err(LoxError::Runtime {
                found: self.to_string(),
                expected: "a number to negate".to_string(),
                line: None,
            }),
        }
    }
}

impl ops::Div for Literal {
    type Output = Result<Literal, LoxError>;

    fn div(self, rhs: Self) -> Self::Output {
        let lhs = self.into_number()?;
        let rhs = rhs.into_number()?;
        Ok(Literal::Number(OrderedFloat(lhs / rhs)))
    }
}

impl ops::Mul for Literal {
    type Output = Result<Literal, LoxError>;

    fn mul(self, rhs: Self) -> Self::Output {
        let lhs = self.into_number()?;
        let rhs = rhs.into_number()?;
        Ok(Literal::Number(OrderedFloat(lhs * rhs)))
    }
}

impl cmp::PartialOrd for Literal {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let us = self.as_number().ok()?;
        let them = other.as_number().ok()?;
        us.partial_cmp(them)
    }
}

impl Literal {
    pub fn into_number(self) -> Result<f64, LoxError> {
        match self {
            Literal::Number(n) => Ok(*n),
            _ => Err(LoxError::Runtime {
                found: self.to_string(),
                expected: "f64".to_string(),
                line: None,
            }),
        }
    }

    fn as_number(&self) -> Result<&f64, LoxError> {
        match self {
            Literal::Number(n) => Ok(n),
            _ => Err(LoxError::Runtime {
                found: self.to_string(),
                expected: "f64".to_string(),
                line: None,
            }),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Literal::Null => false,
            Literal::Boolean(b) => *b,
            _ => true,
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::String(s) => write!(f, "{}", s),
            Literal::Null => write!(f, "nil"),
            Literal::Number(n) => {
                if n.fract() == 0.0 {
                    // Don't print decimal places for integers
                    write!(f, "{}", n.trunc())
                } else {
                    write!(f, "{}", n)
                }
            }
            Literal::Boolean(b) => write!(f, "{}", b),
        }
    }
}

impl From<String> for Literal {
    fn from(v: String) -> Self {
        Literal::String(v)
    }
}

impl From<&str> for Literal {
    fn from(v: &str) -> Self {
        Literal::String(v.to_string())
    }
}

impl From<f64> for Literal {
    fn from(value: f64) -> Self {
        Literal::Number(OrderedFloat(value))
    }
}

impl From<u32> for Literal {
    fn from(value: u32) -> Self {
        Literal::Number(OrderedFloat(value as f64))
    }
}

impl From<()> for Literal {
    fn from(_value: ()) -> Self {
        Literal::Null
    }
}

impl From<bool> for Literal {
    fn from(value: bool) -> Self {
        Literal::Boolean(value)
    }
}
