use crate::expr::{Assign, Binary, BorrowingVisitor, Expr, Grouping, Literal, Unary, Variable};

pub struct AstPrinter {}

impl AstPrinter {
    pub fn print(&self, expr: Expr) -> String {
        expr.accept_borrowed::<String>(self)
    }

    fn parenthesize(&self, name: &str, exprs: &[&Expr]) -> String {
        let parts: Vec<_> = exprs
            .iter()
            .map(|expr| expr.accept_borrowed(self))
            .collect();
        format!("({} {})", name, parts.join(" "))
    }
}

impl BorrowingVisitor<String> for &AstPrinter {
    fn borrow_binary(&self, expr: &Binary) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&*expr.left, &*expr.right])
    }

    fn borrow_grouping(&self, expr: &Grouping) -> String {
        self.parenthesize("group", &[&*expr.expression])
    }

    fn borrow_literal(&self, expr: &Literal) -> String {
        expr.value.to_string()
    }

    fn borrow_unary(&self, expr: &Unary) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&*expr.right])
    }

    fn borrow_variable(&self, expr: &Variable) -> String {
        expr.name.to_string()
    }

    fn borrow_assign(&self, expr: &Assign) -> String {
        self.parenthesize("assign", &[&*expr.value])
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
            Unary::expr(
                Token::new(TokenType::Minus, "-", ().into(), 1),
                Literal::expr(123_f64.into()),
            ),
            Token::new(TokenType::Star, "*", ().into(), 1),
            Grouping::expr(Literal::expr(45.67.into())),
        );
        let printer = AstPrinter {};
        assert_eq!(printer.print(expr), "(* (- 123) (group 45.67))".to_string());
    }
}
