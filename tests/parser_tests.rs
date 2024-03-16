use orca::ast::evaluator::ASTEvaluator;
use orca::ast::symbol::Symbol;
use orca::parser::Parser;

fn assert_expr(expr: &str, expected: f64) {
    let mut evaluator = ASTEvaluator::new();
    let mut acc = 0.0;

    let program = Parser::new(expr).parse();
    for option in evaluator.eval(program) {
        if let Some(Symbol::Number(value)) = option {
            acc += value;
        }
    }

    assert_eq!(expected, acc);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn expressions() {
        assert_expr("2", 2.0);
        assert_expr("1+2", 3.0);
        assert_expr("-2", -2.0);
        assert_expr("1+2*3", 7.0);
        assert_expr("3*2 +1", 7.0);
        assert_expr("2 ^ 3 + 1", 9.0);
        assert_expr("12/2/3", 2.0);
        assert_expr("(1 + 2) * 3", 9.0);
        assert_expr("(-2) ^ 2", 4.0);
        assert_expr("-2 ^ 2", -4.0);
    }

    #[test]
    fn expression_statements() {
        assert_expr("x = 2 * 3\n x+3", 9.0);
    }

    #[test]
    fn functions() {
        assert_expr("func foo() {\nx = 1\nreturn x\n}\nfoo()", 1.0);
        assert_expr(
            "x = 1\nfunc foo(a,b) {\ny = 4\nreturn y + a + b + x\n}\nfoo(x, 100)",
            106.0,
        );
    }

    #[test]
    fn conditionals() {
        assert_expr("x = 10\nif 2 > 1 {\n x = 20\n}\nx", 20.0);
        assert_expr(
            "foo=1\nx = true\ny = false\nif x || y {\n foo = 2\n}\nfoo",
            2.0,
        );
    }
}
