use common::utils::assert_expr;
use orca::ast::symbol::Symbol;

mod common;

#[test]
fn scripts() {
    assert_expr(
        r#"path = "./data/aviation/bland/"
output=ls $path | grep '..tmp'
output"#,
        Symbol::String("gear.tmp\n".to_string()),
    );

    assert_expr(r#"echo "$FOOBAR""#, Symbol::String("\n".to_string()));
    assert_expr(
        "echo '# $FOOBAR'",
        Symbol::String("# $FOOBAR\n".to_string()),
    );
    assert_expr(
        "echo 'foo'; echo 'bar'",
        Symbol::String("foo\nbar\n".to_string()),
    );
    assert_expr("1 > 0 && echo 'foo'", Symbol::String("foo\n".to_string()));
}
