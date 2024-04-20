use common::utils::assert_expr;
use sod::new_string_symbol;

mod common;

#[test]
fn scripts() {
    assert_expr(
        r#"path = "./data/aviation/bland/"
output=ls $path | grep '..tmp'
output"#,
        new_string_symbol!("gear.tmp\n".to_string()),
    );

    assert_expr(
        r#"x = "foo"
echo "$x""#,
        new_string_symbol!("foo\n".to_string()),
    );
    assert_expr(
        "echo '# $FOOBAR'",
        new_string_symbol!("# $FOOBAR\n".to_string()),
    );
    assert_expr(
        "echo 'foo'; echo 'bar'",
        new_string_symbol!("foo\nbar\n".to_string()),
    );
    assert_expr(
        "1 > 0 && echo 'foo'",
        new_string_symbol!("foo\n".to_string()),
    );
}
