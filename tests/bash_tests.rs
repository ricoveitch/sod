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
    )
}
