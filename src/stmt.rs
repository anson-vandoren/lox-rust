use crate::{expr::Expr, token::Token};

#[derive(Debug)]
pub struct Expression {
    pub expression: Expr,
}

impl Expression {
    pub fn stmt(expression: Expr) -> Stmt {
        Stmt::Expression(Self { expression })
    }

    fn accept<T>(self, mut visitor: impl Visitor<T>) -> T {
        visitor.visit_expression_stmt(self)
    }

    fn accept_borrowed<T>(&self, mut visitor: impl BorrowingVisitor<T>) -> T {
        visitor.borrow_expression_stmt(self)
    }
}

#[derive(Debug)]
pub struct Print {
    pub expression: Expr,
}

impl Print {
    pub fn stmt(value: Expr) -> Stmt {
        Stmt::Print(Self { expression: value })
    }

    fn accept<T>(self, mut visitor: impl Visitor<T>) -> T {
        visitor.visit_print_stmt(self)
    }

    fn accept_borrowed<T>(&self, mut visitor: impl BorrowingVisitor<T>) -> T {
        visitor.borrow_print_stmt(self)
    }
}

#[derive(Debug)]
pub struct Var {
    pub name: Token,
    pub initializer: Option<Expr>,
}

impl Var {
    pub fn stmt(name: Token, initializer: Option<Expr>) -> Stmt {
        Stmt::Var(Self { name, initializer })
    }

    fn accept<T>(self, mut visitor: impl Visitor<T>) -> T {
        visitor.visit_var_stmt(self)
    }

    fn accept_borrowed<T>(&self, mut visitor: impl BorrowingVisitor<T>) -> T {
        visitor.borrow_var_stmt(self)
    }
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

impl Block {
    pub fn stmt(statements: Vec<Stmt>) -> Stmt {
        Stmt::Block(Self { statements })
    }

    fn accept<T>(self, mut visitor: impl Visitor<T>) -> T {
        visitor.visit_block_stmt(self)
    }

    fn accept_borrowed<T>(&self, mut visitor: impl BorrowingVisitor<T>) -> T {
        visitor.borrow_block_stmt(self)
    }
}

#[derive(Debug)]
pub struct If {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug)]
pub struct While {
    pub condition: Expr,
    pub body: Box<Stmt>,
}

impl While {
    pub fn stmt(condition: Expr, body: Stmt) -> Stmt {
        Stmt::While(Self {
            condition,
            body: Box::new(body),
        })
    }

    fn accept<T>(self, mut visitor: impl Visitor<T>) -> T {
        visitor.visit_while_stmt(self)
    }

    fn accept_borrowed<T>(&self, mut visitor: impl BorrowingVisitor<T>) -> T {
        visitor.borrow_while_stmt(self)
    }
}

impl If {
    pub fn stmt(condition: Expr, then_branch: Stmt, else_branch: Option<Stmt>) -> Stmt {
        let else_branch = else_branch.map(Box::new);
        Stmt::If(Self {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn accept<T>(self, mut visitor: impl Visitor<T>) -> T {
        visitor.visit_if_stmt(self)
    }

    fn accept_borrowed<T>(&self, mut visitor: impl BorrowingVisitor<T>) -> T {
        visitor.borrow_if_stmt(self)
    }
}

pub enum Stmt {
    Block(Block),
    Expression(Expression),
    Print(Print),
    Var(Var),
    If(If),
    While(While),
}

impl std::fmt::Debug for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Block(stmt) => write!(f, "{:?}", stmt),
            Self::Expression(stmt) => write!(f, "{:?}", stmt),
            Self::Print(stmt) => write!(f, "{:?}", stmt),
            Self::Var(stmt) => write!(f, "{:?}", stmt),
            Self::If(stmt) => write!(f, "{:?}", stmt),
            Self::While(stmt) => write!(f, "{:?}", stmt),
        }
    }
}

impl Stmt {
    pub fn accept<T>(self, visitor: impl Visitor<T>) -> T {
        match self {
            Self::Block(stmt) => stmt.accept(visitor),
            Self::Expression(stmt) => stmt.accept(visitor),
            Self::Print(stmt) => stmt.accept(visitor),
            Self::Var(stmt) => stmt.accept(visitor),
            Self::If(stmt) => stmt.accept(visitor),
            Self::While(stmt) => stmt.accept(visitor),
        }
    }

    pub fn accept_borrowed<T>(&self, visitor: impl BorrowingVisitor<T>) -> T {
        match self {
            Self::Block(stmt) => stmt.accept_borrowed(visitor),
            Self::Expression(stmt) => stmt.accept_borrowed(visitor),
            Self::Print(stmt) => stmt.accept_borrowed(visitor),
            Self::Var(stmt) => stmt.accept_borrowed(visitor),
            Self::If(stmt) => stmt.accept_borrowed(visitor),
            Self::While(stmt) => stmt.accept_borrowed(visitor),
        }
    }
}

pub trait Visitor<T> {
    fn visit_block_stmt(&mut self, stmt: Block) -> T;
    fn visit_expression_stmt(&mut self, stmt: Expression) -> T;
    fn visit_print_stmt(&mut self, stmt: Print) -> T;
    fn visit_var_stmt(&mut self, stmt: Var) -> T;
    fn visit_if_stmt(&mut self, stmt: If) -> T;
    fn visit_while_stmt(&mut self, stmt: While) -> T;
}

pub trait BorrowingVisitor<T> {
    fn borrow_block_stmt(&mut self, stmt: &Block) -> T;
    fn borrow_expression_stmt(&mut self, stmt: &Expression) -> T;
    fn borrow_print_stmt(&mut self, stmt: &Print) -> T;
    fn borrow_var_stmt(&mut self, stmt: &Var) -> T;
    fn borrow_if_stmt(&mut self, stmt: &If) -> T;
    fn borrow_while_stmt(&mut self, stmt: &While) -> T;
}
