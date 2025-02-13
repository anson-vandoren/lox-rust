#[derive(Clone, Debug)]
pub enum Object {
    String(String),
    None,
    Number(f64),
    Boolean(bool),
}

impl From<String> for Object {
    fn from(value: String) -> Self {
        Object::String(value)
    }
}

impl From<f64> for Object {
    fn from(value: f64) -> Self {
        Object::Number(value)
    }
}

impl From<()> for Object {
    fn from(_value: ()) -> Self {
        Object::None
    }
}

impl From<bool> for Object {
    fn from(value: bool) -> Self {
        Object::Boolean(value)
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", s),
            Self::None => write!(f, "nil"),
            Self::Number(n) => write!(f, "{}", n),
            Self::Boolean(b) => write!(f, "{}", b),
        }
    }
}
