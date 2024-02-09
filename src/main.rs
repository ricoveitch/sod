use orca::ast::visitor;
use orca::parser::Parser;
use std::io::{self, Write};

fn main() {
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        let ast = Parser::new(&buffer).parse();
        println!("{}", visitor::visit(ast));
    }
}
