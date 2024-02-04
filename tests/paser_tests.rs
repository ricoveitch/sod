use orca::{ast::visitor, parser::Parser};

fn assert_expr(expr: &str, expected: f64) {
    let mut parser = Parser::new(expr);
    assert_eq!(expected, visitor::visit(parser.parse()));
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parser() {
        assert_expr("2", 2.0);
        assert_expr("1+2*3", 7.0);
        assert_expr("3*2 +1", 7.0);
        assert_expr("2 ^ 3 + 1", 9.0);
        assert_expr("12/2/3", 2.0);
    }
}
