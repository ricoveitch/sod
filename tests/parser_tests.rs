use common::utils::{assert_expr, assert_exprs, eval_expr};
use orca::new_string_symbol;
use orca::symbol::symbol::Symbol;
mod common;

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

#[should_panic]
#[test]
fn invalid_number() {
    eval_expr("1.");
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
    assert_expr(
        "
        x=1
        y=0
        if x == 1 {
            t = 1
            if y == 0 {
                z = 1
            }
            x = t + 1
        }
        x",
        Symbol::Number(2.0),
    );
    assert_expr("1 || echo 'foo'", Symbol::Number(1.0));
    assert_expr("none && 1", Symbol::None);
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
        new_string_symbol!("foo".to_string()),
    );
    assert_expr(
        r#"x = "foo"
x + "bar"
"#,
        new_string_symbol!("foobar".to_string()),
    );
    assert_expr(
        r#"x = "foo"
        x.len()
        "#,
        Symbol::Number(3.0),
    );
    assert_expr(
        r#"x = "abc"
        x[1]
        "#,
        new_string_symbol!("b".to_string()),
    );
    assert_expr(
        r#"x = "foo"
        x.push("bar")
        x
        "#,
        new_string_symbol!("foobar".to_string()),
    );
    assert_exprs(
        vec![
            r#"x = "abc"
            x.pop()
            "#,
            "x",
            r#"x.insert(0,"z")
                x"#,
            "x.remove(1)",
            "x",
        ],
        vec![
            new_string_symbol!("c".to_string()),
            new_string_symbol!("ab".to_string()),
            new_string_symbol!("zab".to_string()),
            new_string_symbol!("a".to_string()),
            new_string_symbol!("zb".to_string()),
        ],
    );
}

#[test]
fn lists() {
    assert_expr("x = [1, 2]\nx[1]", Symbol::Number(2.0));
    assert_expr("x = []\nx.push(5)\nx.push(6)\nx.pop()", Symbol::Number(6.0));
    assert_expr("x = [5]\nx[0] = 1\nx[0]", Symbol::Number(1.0));
    assert_expr("x = [5]\nx_0 = x[0]\nx_0 = 1\nx[0]", Symbol::Number(5.0));
    assert_expr("x = [1,2,3]\nx.remove(1)\nx[1]", Symbol::Number(3.0));
    assert_expr("x = [1,2]\nx.insert(1,4)\nx[1]", Symbol::Number(4.0));
    assert_expr(
        "t = 0\nx = [5,2]\nfor v in x {\nt = t + v\n}\nt",
        Symbol::Number(7.0),
    );
}

#[test]
fn ranges() {
    assert_expr(
        "t = 0\nfor v in 1..3 {\n t = t + v\n}\nt",
        Symbol::Number(3.0),
    );
    assert_expr(
        "t = 0\nfor v in 1..4..2 {\n t = t + v\n}\nt",
        Symbol::Number(4.0),
    );
    assert_expr(
        "t = 0\nfor v in 4..1..-1 {\n t = t + v\n}\nt",
        Symbol::Number(9.0),
    );
}

#[test]
fn global_vars() {
    assert_expr("process.argv.len()", Symbol::Number(0.0));
}
