use crate::expr::{Binary, Expr, Grouping, Literal, Unary, Visitor};

pub struct AstPrinter {}

impl AstPrinter {
    pub fn print(&self, expr: Expr) -> String {
        expr.accept::<String>(self)
    }

    fn parenthesize(&self, name: &str, exprs: &[&Expr]) -> String {
        let parts: Vec<_> = exprs.iter().map(|expr| expr.accept(self)).collect();
        format!("({} {})", name, parts.join(" "))
    }
}

impl Visitor<String> for &AstPrinter {
    fn visit_binary(&self, expr: &Binary) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&*expr.left, &*expr.right])
    }

    fn visit_grouping(&self, expr: &Grouping) -> String {
        self.parenthesize("group", &[&*expr.expression])
    }

    fn visit_literal(&self, expr: &Literal) -> String {
        expr.value.to_string()
    }

    fn visit_unary(&self, expr: &Unary) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&*expr.right])
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
