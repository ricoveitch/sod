use orca::ast;
use orca::parser::Parser;
use std::io::{self, Write};

fn main() {
    let mut evaluator = ast::evaluator::ASTEvaluator::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        let program = Parser::new(&buffer).parse();
        println!("ast:{:?}", program);
        for option in evaluator.eval(program) {
            if let Some(value) = option {
                println!("{}", value);
            }
        }
    }
}
