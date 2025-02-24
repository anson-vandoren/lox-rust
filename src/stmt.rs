use crate::{expr::Expr, token::Token};

#[derive(Clone)]
pub struct Expression {
    pub expression: Expr,
}

impl std::fmt::Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.expression)
    }
}

impl Expression {
    pub fn stmt(expression: Expr) -> Stmt {
        Stmt::Expression(Self { expression })
    }
}

#[derive(Clone)]
pub struct Print {
    pub expression: Expr,
}

impl std::fmt::Debug for Print {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "print {:?};", &self.expression)
    }
}

impl Print {
    pub fn stmt(value: Expr) -> Stmt {
        Stmt::Print(Self { expression: value })
    }
}

#[derive(Clone, Debug)]
pub struct Var {
    pub name: Token,
    pub initializer: Option<Expr>,
}

// impl std::fmt::Debug for Var {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        let val = if let Some(init) = &self.initializer {
//            format!("{:?}", init)
//        } else {
//            "nil".to_string()
//        };
//        write!(f, "VarStmt({} = {})", self.name.lexeme, val)
//    }
//}

impl Var {
    pub fn stmt(name: Token, initializer: Option<Expr>) -> Stmt {
        Stmt::Var(Self { name, initializer })
    }
}

#[derive(Clone)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

impl std::fmt::Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block{:?}", self.statements)
    }
}

impl Block {
    pub fn stmt(statements: Vec<Stmt>) -> Stmt {
        Stmt::Block(Self { statements })
    }
}

#[derive(Clone, Debug)]
pub struct If {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Clone)]
pub struct While {
    pub condition: Expr,
    pub body: Box<Stmt>,
}

impl std::fmt::Debug for While {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "While ({:?}) {{ {:?} }}", &self.condition, &self.body)
    }
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

#[derive(Clone, Debug)]
pub struct Function {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

impl Function {
    pub fn stmt(name: Token, params: Vec<Token>, body: Vec<Stmt>) -> Stmt {
        Stmt::Function(Self { name, params, body })
    }
}

#[derive(Clone)]
pub struct Return {
    pub keyword: Token,
    pub value: Option<Expr>,
}

impl std::fmt::Debug for Return {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = if let Some(ret) = &self.value {
            format!("{:?}", ret)
        } else {
            "nil".to_string()
        };
        write!(f, "return {:?};", val)
    }
}

impl Return {
    pub fn stmt(keyword: Token, value: Option<Expr>) -> Stmt {
        Stmt::Return(Self { keyword, value })
    }
}

#[derive(Clone, Debug)]
pub struct Class {
    pub name: Token,
    pub methods: Vec<Function>,
}

impl Class {
    pub fn stmt(name: Token, methods: Vec<Function>) -> Stmt {
        Stmt::Class(Self { name, methods })
    }
}

#[derive(Clone)]
pub enum Stmt {
    Block(Block),
    Expression(Expression),
    Print(Print),
    Var(Var),
    If(If),
    While(While),
    Function(Function),
    Return(Return),
    Class(Class),
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
            Self::Return(stmt) => write!(f, "{:?}", stmt),
            Self::Class(stmt) => write!(f, "{:?}", stmt),
        }
    }
}
