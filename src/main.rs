use orca::parser::Parser;

fn main() {
    let mut parser = Parser::new("3*2+1");
    let ast = parser.parse();
    println!("{:?}", ast);
}
