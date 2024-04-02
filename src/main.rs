use orca::ast::evaluator::ASTEvaluator;
use orca::parser::Parser;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

fn parse_file(filename: &str) {
    let src = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("failed to read file: {}", err.to_string());
            process::exit(1);
        }
    };

    let ast = Parser::new(&src).parse();
    let mut evaluator = ASTEvaluator::new();
    evaluator.eval(ast);
}

fn interpret() {
    let mut evaluator = ASTEvaluator::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        let program = Parser::new(&buffer).parse();
        for option in evaluator.eval(program) {
            if let Some(value) = option {
                println!("{}", value);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: {} [filename]", args[0]);
        std::process::exit(1);
    }

    match args.get(1) {
        Some(filename) => parse_file(filename),
        None => interpret(),
    };
}
