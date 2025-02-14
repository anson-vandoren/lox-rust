use macros::ExpressionType;

use crate::{object::Object, token::Token};

#[derive(Debug, ExpressionType)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, ExpressionType)]
pub struct Logical {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, ExpressionType)]
pub struct Grouping {
    pub expression: Box<Expr>,
}

#[derive(Debug, ExpressionType)]
pub struct Literal {
    pub value: Object,
}

#[derive(Debug, ExpressionType)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, ExpressionType)]
pub struct Variable {
    pub name: Token,
}

#[derive(Debug, ExpressionType)]
pub struct Assign {
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Debug)]
pub enum Expr {
    Binary(Binary),
    Logical(Logical),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Variable(Variable),
    Assign(Assign),
}

impl Expr {
    pub fn accept<T>(self, visitor: impl Visitor<T>) -> T {
        match self {
            Self::Binary(expr) => expr.accept(visitor),
            Self::Logical(expr) => expr.accept(visitor),
            Self::Grouping(expr) => expr.accept(visitor),
            Self::Literal(expr) => expr.accept(visitor),
            Self::Unary(expr) => expr.accept(visitor),
            Self::Variable(expr) => expr.accept(visitor),
            Self::Assign(expr) => expr.accept(visitor),
        }
    }

    pub fn accept_borrowed<T>(&self, visitor: impl BorrowingVisitor<T>) -> T {
        match self {
            Self::Binary(expr) => expr.accept_borrowed(visitor),
            Self::Logical(expr) => expr.accept_borrowed(visitor),
            Self::Grouping(expr) => expr.accept_borrowed(visitor),
            Self::Literal(expr) => expr.accept_borrowed(visitor),
            Self::Unary(expr) => expr.accept_borrowed(visitor),
            Self::Variable(expr) => expr.accept_borrowed(visitor),
            Self::Assign(expr) => expr.accept_borrowed(visitor),
        }
    }
}

pub trait Visitor<T> {
    fn visit_binary(&mut self, expr: Binary) -> T;
    fn visit_logical(&mut self, expr: Logical) -> T;
    fn visit_grouping(&mut self, expr: Grouping) -> T;
    fn visit_literal(&self, expr: Literal) -> T;
    fn visit_unary(&mut self, expr: Unary) -> T;
    fn visit_variable(&self, expr: Variable) -> T;
    fn visit_assign(&mut self, expr: Assign) -> T;
}

pub trait BorrowingVisitor<T> {
    fn borrow_binary(&mut self, expr: &Binary) -> T;
    fn borrow_logical(&mut self, expr: &Logical) -> T;
    fn borrow_grouping(&mut self, expr: &Grouping) -> T;
    fn borrow_literal(&mut self, expr: &Literal) -> T;
    fn borrow_unary(&mut self, expr: &Unary) -> T;
    fn borrow_variable(&mut self, expr: &Variable) -> T;
    fn borrow_assign(&mut self, expr: &Assign) -> T;
}
