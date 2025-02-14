use crate::{expr::Expr, token::Token};

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
}

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
}

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
}

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
}

pub struct If {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

impl If {
    pub fn stmt(condition: Expr, then_branch: Stmt, else_branch: Option<Stmt>) -> Stmt {
        let else_branch = if let Some(branch) = else_branch {
            Some(Box::new(branch))
        } else {
            None
        };
        Stmt::If(Self {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn accept<T>(self, mut visitor: impl Visitor<T>) -> T {
        visitor.visit_if_stmt(self)
    }
}

pub enum Stmt {
    Block(Block),
    Expression(Expression),
    Print(Print),
    Var(Var),
    If(If),
}

impl Stmt {
    pub fn accept<T>(self, visitor: impl Visitor<T>) -> T {
        match self {
            Self::Block(stmt) => stmt.accept(visitor),
            Self::Expression(stmt) => stmt.accept(visitor),
            Self::Print(stmt) => stmt.accept(visitor),
            Self::Var(stmt) => stmt.accept(visitor),
            Self::If(stmt) => stmt.accept(visitor),
        }
    }
}

pub trait Visitor<T> {
    fn visit_block_stmt(&mut self, stmt: Block) -> T;
    fn visit_expression_stmt(&mut self, stmt: Expression) -> T;
    fn visit_print_stmt(&mut self, stmt: Print) -> T;
    fn visit_var_stmt(&mut self, stmt: Var) -> T;
    fn visit_if_stmt(&mut self, stmt: If) -> T;
}
