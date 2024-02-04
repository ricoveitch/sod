use orca::ast::visitor;
use orca::parser::Parser;

fn main() {
    let mut parser = Parser::new("1+2*3");
    let ast = parser.parse();
    println!("{:?}", ast);
    let r = visitor::visit(ast);
    println!("r={}", r);
}
