use macros::ExpressionType;

use crate::{object::Object, token::Token};

#[derive(Clone, Debug, Eq, ExpressionType, Hash, PartialEq)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug, Eq, ExpressionType, Hash, PartialEq)]
pub struct Logical {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug, Eq, ExpressionType, Hash, PartialEq)]
pub struct Grouping {
    pub expression: Box<Expr>,
}

#[derive(Clone, Debug, Eq, ExpressionType, Hash, PartialEq)]
pub struct Literal {
    pub value: Object,
}

#[derive(Clone, Debug, Eq, ExpressionType, Hash, PartialEq)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug, Eq, ExpressionType, Hash, PartialEq)]
pub struct Variable {
    pub name: Token,
}

#[derive(Clone, Debug, Eq, ExpressionType, Hash, PartialEq)]
pub struct Assign {
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Clone, Debug, Eq, ExpressionType, Hash, PartialEq)]
pub struct Call {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Expr {
    Binary(Binary),
    Logical(Logical),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Variable(Variable),
    Assign(Assign),
    Call(Call),
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binary(expr) => write!(f, "{:?}", expr),
            Self::Logical(expr) => write!(f, "{:?}", expr),
            Self::Grouping(expr) => write!(f, "{:?}", expr),
            Self::Literal(expr) => write!(f, "{:?}", expr),
            Self::Unary(expr) => write!(f, "{:?}", expr),
            Self::Variable(expr) => write!(f, "{:?}", expr),
            Self::Assign(expr) => write!(f, "{:?}", expr),
            Self::Call(expr) => write!(f, "{:?}", expr),
        }
    }
}
