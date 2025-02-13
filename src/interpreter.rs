use crate::{expr::Visitor, object::Object};

pub struct Interpreter {}

impl Visitor<Object> for &Interpreter {
    fn visit_binary(&self, expr: &crate::expr::Binary) -> Object {
        todo!()
    }

    fn visit_grouping(&self, expr: &crate::expr::Grouping) -> Object {
        todo!()
    }

    fn visit_literal(&self, expr: &crate::expr::Literal) -> Object {
        expr.value.clone()
    }

    fn visit_unary(&self, expr: &crate::expr::Unary) -> Object {
        todo!()
    }
}
