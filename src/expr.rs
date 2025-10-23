use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

use macros::ExpressionType;
use tracing::trace;

use crate::token::Token;

#[derive(Clone, ExpressionType)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

impl std::fmt::Debug for Binary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?} {:?}", self.left, self.operator, self.right)
    }
}

#[derive(Clone, Debug, ExpressionType)]
pub struct Logical {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug, ExpressionType)]
pub struct Grouping {
    pub expression: Box<Expr>,
}

#[derive(Clone, ExpressionType)]
pub struct Literal {
    pub value: crate::object::Literal,
}

impl std::fmt::Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Literal({:?})", self.value)
    }
}

#[derive(Clone, Debug, ExpressionType)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

static COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: Token,
}

impl Variable {
    pub fn expr(mut name: Token) -> Expr {
        // Token is used as the key for locals, needs to be unique to _this_ instance of the
        // variable being referenced to make sure scopes are correct
        let nonce = COUNTER.fetch_add(1, Relaxed);
        name.literal = nonce.into();
        trace!(?name, nonce, "Creating variable");
        Expr::Variable(Self { name })
    }
}

#[derive(Clone, ExpressionType)]
pub struct Assign {
    pub name: Token,
    pub value: Box<Expr>,
}

impl std::fmt::Debug for Assign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Assign({} = {:?};)", self.name.lexeme, self.value)
    }
}

#[derive(Clone, Debug, ExpressionType)]
pub struct Call {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

#[derive(Clone, ExpressionType)]
pub struct Get {
    pub object: Box<Expr>,
    pub name: Token,
}

impl std::fmt::Debug for Get {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let obj_name = match *self.object.clone() {
            Expr::Variable(var) => var.name.lexeme,
            o => format!("{:?}", o),
        };
        let member = self.name.lexeme.clone();
        write!(f, "{obj_name}.{member}")
    }
}

#[derive(Clone, Debug, ExpressionType)]
pub struct Set {
    pub object: Box<Expr>,
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Clone, Debug, ExpressionType)]
pub struct This {
    pub keyword: Token,
}

#[derive(Clone)]
pub enum Expr {
    Binary(Binary),
    Logical(Logical),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Variable(Variable),
    Assign(Assign),
    Call(Call),
    Get(Get),
    Set(Set),
    This(This),
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
            Self::Get(expr) => write!(f, "{:?}", expr),
            Self::Set(expr) => write!(f, "{:?}", expr),
            Self::This(expr) => write!(f, "{:?}", expr),
        }
    }
}
