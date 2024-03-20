use orca::ast::evaluator::ASTEvaluator;
use orca::ast::symbol::Symbol;
use orca::parser::Parser;

fn eval_expr(expr: &str) -> Vec<Option<Symbol>> {
    let mut evaluator = ASTEvaluator::new();
    let program = Parser::new(expr).parse();
    evaluator.eval(program)
}

fn assert_expr(expr: &str, expected: Symbol) {
    let evaluation = eval_expr(expr);
    let symbol = evaluation.last().unwrap().as_ref().unwrap();
    assert_eq!(&expected, symbol);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn math_expressions() {
        assert_expr("2", Symbol::Number(2.0));
        assert_expr("1+2", Symbol::Number(3.0));
        assert_expr("-2", Symbol::Number(-2.0));
        assert_expr("1+2*3", Symbol::Number(7.0));
        assert_expr("3*2 +1", Symbol::Number(7.0));
        assert_expr("2 ^ 3 + 1", Symbol::Number(9.0));
        assert_expr("12/2/3", Symbol::Number(2.0));
        assert_expr("(1 + 2) * 3", Symbol::Number(9.0));
        assert_expr("(-2) ^ 2", Symbol::Number(4.0));
        assert_expr("-2 ^ 2", Symbol::Number(-4.0));
    }

    #[test]
    fn math_expression_statements() {
        assert_expr("x = 2 * 3\n x+3", Symbol::Number(9.0));
    }

    #[test]
    fn functions() {
        assert_expr(
            "func foo() {\nx = 1\nreturn x\n}\nfoo()",
            Symbol::Number(1.0),
        );
        assert_expr(
            "x = 1\nfunc foo(a,b) {\ny = 4\nreturn y + a + b + x\n}\nfoo(x, 100)",
            Symbol::Number(106.0),
        );
    }

    #[test]
    fn conditionals() {
        assert_expr("x = 10\nif 2 > 1 {\n x = 20\n}\nx", Symbol::Number(20.0));
        assert_expr("x = 10\nif 2 > 1 {\n x = 20\n}\nx", Symbol::Number(20.0));
        assert_expr(
            "foo=1\nx = true\ny = false\nif x || y {\n foo = 2\n}\nfoo",
            Symbol::Number(2.0),
        );
        assert_expr(
            "x=1\nif x != 1 {\n x = 2\n} else {\n x=3\n}\nx",
            Symbol::Number(3.0),
        );
        assert_expr(
            "x=1\nif x != 1 {\n x = 2\n} else {\n x=3\n}\nif x == 3 {\n x = 4\n}\nx",
            Symbol::Number(4.0),
        );
        assert_expr(
            "
        x=1
        y=2
        if x == 1 {
            if y == 0 {
                x = 10
            } else {
                if x == 1 {
                    x = 20
                }
            }
        }
        x",
            Symbol::Number(20.0),
        );
    }

    #[test]
    #[should_panic]
    fn conditional_var_panic() {
        eval_expr("x=1\nif x != 1 {\n x = 2\n} else {\n y=5\nx=3\n}\ny");
    }

    #[test]
    fn strings() {
        assert_expr(
            r#"x="foo"
                    x"#,
            Symbol::String("foo".to_string()),
        );
    }
}
