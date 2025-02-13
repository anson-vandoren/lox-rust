use crate::{object::Object, token::Token};

pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

impl Binary {
    pub fn expr(left: Expr, operator: Token, right: Expr) -> Expr {
        Expr::Binary(Self::new(left, operator, right))
    }

    pub fn new(left: Expr, operator: Token, right: Expr) -> Self {
        Self {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    fn accept<T>(self, visitor: impl Visitor<T>) -> T {
        visitor.visit_binary(self)
    }

    fn accept_borrowed<T>(&self, visitor: impl BorrowingVisitor<T>) -> T {
        visitor.borrow_binary(self)
    }
}

pub struct Grouping {
    pub expression: Box<Expr>,
}

impl Grouping {
    pub fn expr(expr: Expr) -> Expr {
        Expr::Grouping(Self::new(expr))
    }

    pub fn new(expr: Expr) -> Grouping {
        Self {
            expression: Box::new(expr),
        }
    }

    fn accept<T>(self, visitor: impl Visitor<T>) -> T {
        visitor.visit_grouping(self)
    }

    fn accept_borrowed<T>(&self, visitor: impl BorrowingVisitor<T>) -> T {
        visitor.borrow_grouping(self)
    }
}

pub struct Literal {
    pub value: Object,
}

impl Literal {
    pub fn expr(value: Object) -> Expr {
        Expr::Literal(Self::new(value))
    }

    pub fn new(value: Object) -> Literal {
        Self { value }
    }

    fn accept<T>(self, visitor: impl Visitor<T>) -> T {
        visitor.visit_literal(self)
    }

    fn accept_borrowed<T>(&self, visitor: impl BorrowingVisitor<T>) -> T {
        visitor.borrow_literal(self)
    }
}

pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

impl Unary {
    pub fn expr(operator: Token, right: Expr) -> Expr {
        Expr::Unary(Self::new(operator, right))
    }

    pub fn new(operator: Token, right: Expr) -> Unary {
        Self {
            operator,
            right: Box::new(right),
        }
    }

    fn accept<T>(self, visitor: impl Visitor<T>) -> T {
        visitor.visit_unary(self)
    }

    fn accept_borrowed<T>(&self, visitor: impl BorrowingVisitor<T>) -> T {
        visitor.borrow_unary(self)
    }
}

pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
}

impl Expr {
    pub fn accept<T>(self, visitor: impl Visitor<T>) -> T {
        match self {
            Self::Binary(expr) => expr.accept(visitor),
            Self::Grouping(expr) => expr.accept(visitor),
            Self::Literal(expr) => expr.accept(visitor),
            Self::Unary(expr) => expr.accept(visitor),
        }
    }

    pub fn accept_borrowed<T>(&self, visitor: impl BorrowingVisitor<T>) -> T {
        match self {
            Self::Binary(expr) => expr.accept_borrowed(visitor),
            Self::Grouping(expr) => expr.accept_borrowed(visitor),
            Self::Literal(expr) => expr.accept_borrowed(visitor),
            Self::Unary(expr) => expr.accept_borrowed(visitor),
        }
    }
}

pub trait Visitor<T> {
    fn visit_binary(&self, binary: Binary) -> T;
    fn visit_grouping(&self, grouping: Grouping) -> T;
    fn visit_literal(&self, literal: Literal) -> T;
    fn visit_unary(&self, unary: Unary) -> T;
}

pub trait BorrowingVisitor<T> {
    fn borrow_binary(&self, binary: &Binary) -> T;
    fn borrow_grouping(&self, grouping: &Grouping) -> T;
    fn borrow_literal(&self, literal: &Literal) -> T;
    fn borrow_unary(&self, unary: &Unary) -> T;
}
