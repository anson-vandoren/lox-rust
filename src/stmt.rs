use crate::{expr::Expr, token::Token};

#[derive(Debug)]
pub struct Expression {
    pub expression: Expr,
}

impl Expression {
    pub fn stmt(expression: Expr) -> Stmt {
        Stmt::Expression(Self { expression })
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
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

impl Block {
    pub fn stmt(statements: Vec<Stmt>) -> Stmt {
        Stmt::Block(Self { statements })
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
}

#[derive(Debug)]
pub struct Function {
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,
}

impl Function {
    pub fn stmt(name: Token, params: Vec<Token>, body: Vec<Stmt>) -> Stmt {
        Stmt::Function(Self { name, params, body })
    }
}

pub enum Stmt {
    Block(Block),
    Expression(Expression),
    Print(Print),
    Var(Var),
    If(If),
    While(While),
    Function(Function),
}

impl std::fmt::Debug for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Block(stmt) => write!(f, "{:?}", stmt),
            Self::Expression(stmt) => write!(f, "{:?}", stmt),
            Self::Function(stmt) => write!(f, "{:?}", stmt),
            Self::If(stmt) => write!(f, "{:?}", stmt),
            Self::Print(stmt) => write!(f, "{:?}", stmt),
            Self::Var(stmt) => write!(f, "{:?}", stmt),
            Self::While(stmt) => write!(f, "{:?}", stmt),
        }
    }
}
