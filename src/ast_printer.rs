use crate::expr::{Assign, Binary, Expr, Grouping, Literal, Unary, Variable};

pub struct AstPrinter {}

impl AstPrinter {
    #[allow(unused)]
    pub fn print(&self, expr: &Expr) -> String {
        match expr {
            Expr::Binary(expr) => self.print_binary(expr),
            Expr::Logical(expr) => self.print_logical(expr),
            Expr::Grouping(expr) => self.print_grouping(expr),
            Expr::Literal(expr) => self.print_literal(expr),
            Expr::Unary(expr) => self.print_unary(expr),
            Expr::Variable(expr) => self.print_variable(expr),
            Expr::Assign(expr) => self.print_assign(expr),
            Expr::Call(expr) => self.print_call(expr),
            Expr::Get(expr) => self.print_get(expr),
            Expr::Set(set) => todo!(),
            Expr::This(this) => todo!(),
        }
    }

    fn parenthesize(&self, name: &str, exprs: &[&Expr]) -> String {
        let parts: Vec<_> = exprs.iter().map(|expr| self.print(expr)).collect();
        format!("({} {})", name, parts.join(" "))
    }

    fn print_binary(&self, expr: &Binary) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&*expr.left, &*expr.right])
    }

    fn print_logical(&self, expr: &crate::expr::Logical) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&*expr.left, &*expr.right])
    }

    fn print_grouping(&self, expr: &Grouping) -> String {
        self.parenthesize("group", &[&*expr.expression])
    }

    fn print_literal(&self, expr: &Literal) -> String {
        expr.value.to_string()
    }

    fn print_unary(&self, expr: &Unary) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&*expr.right])
    }

    fn print_variable(&self, expr: &Variable) -> String {
        expr.name.to_string()
    }

    fn print_assign(&self, expr: &Assign) -> String {
        self.parenthesize("assign", &[&*expr.value])
    }

    fn print_call(&self, _expr: &crate::expr::Call) -> String {
        todo!()
    }

    fn print_get(&self, _expr: &crate::expr::Get) -> String {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        expr::{Binary, Grouping, Literal, Unary},
        token::Token,
        token_type::TokenType,
    };

    #[test]
    fn does_the_thing() {
        let expr = Binary::expr(
            Unary::expr(Token::new(TokenType::Minus, "-", ().into(), 1), Literal::expr(123_f64.into())),
            Token::new(TokenType::Star, "*", ().into(), 1),
            Grouping::expr(Literal::expr(45.67.into())),
        );
        let printer = AstPrinter {};
        assert_eq!(printer.print(&expr), "(* (- 123) (group 45.67))".to_string());
    }
}
