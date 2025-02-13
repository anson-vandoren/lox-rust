use crate::expr::Expr;

pub struct Expression {
    expression: Expr,
}

impl Expression {
    pub fn stmt(expr: Expr) -> Stmt {
        Stmt::Expression(Self { expression: expr })
    }

    fn accept<T>(self, visitor: impl Visitor<T>) -> T {
        visitor.visit_expression_stmt(self)
    }

    fn accept_borrowed<T>(&self, visitor: impl BorrowingVisitor<T>) -> T {
        visitor.borrow_expression_stmt(self)
    }
}

pub struct Print {}

impl Print {
    pub fn stmt() -> Stmt {
        Stmt::Print(Self {})
    }

    fn accept<T>(self, visitor: impl Visitor<T>) -> T {
        visitor.visit_print_stmt(self)
    }

    fn accept_borrowed<T>(&self, visitor: impl BorrowingVisitor<T>) -> T {
        visitor.borrow_print_stmt(self)
    }
}

pub enum Stmt {
    Expression(Expression),
    Print(Print),
}

impl Stmt {
    pub fn accept<T>(self, visitor: impl Visitor<T>) -> T {
        match self {
            Self::Expression(stmt) => stmt.accept(visitor),
            Self::Print(stmt) => stmt.accept(visitor),
        }
    }

    pub fn accept_borrowed<T>(&self, visitor: impl BorrowingVisitor<T>) -> T {
        match self {
            Self::Expression(stmt) => stmt.accept_borrowed(visitor),
            Self::Print(stmt) => stmt.accept_borrowed(visitor),
        }
    }
}

pub trait Visitor<T> {
    fn visit_expression_stmt(&self, stmt: Expression) -> T;
    fn visit_print_stmt(&self, stmt: Print) -> T;
}

pub trait BorrowingVisitor<T> {
    fn borrow_expression_stmt(&self, stmt: &Expression) -> T;
    fn borrow_print_stmt(&self, stmt: &Print) -> T;
}
