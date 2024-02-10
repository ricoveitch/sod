use orca::ast;
use orca::parser::Parser;
use std::io::{self, Write};

fn main() {
    let mut evaluator = ast::evaluator::ASTVEvaluator::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        let program = Parser::new(&buffer).parse();
        for line in program {
            println!("ast:{:?}", line);
            println!("eval:{}", evaluator.eval(line));
        }
    }
}
