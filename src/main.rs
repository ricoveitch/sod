use sod::ast::evaluator::ASTEvaluator;
use sod::parser::Parser;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

fn get_argv(env_args: Vec<String>) -> Vec<String> {
    let mut argv = env_args.clone();
    argv.remove(0);
    argv
}

fn parse_file(env_args: Vec<String>) {
    let argv = get_argv(env_args);
    let filename = argv.get(0).unwrap();
    let src = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("failed to read file: {}", err.to_string());
            process::exit(1);
        }
    };

    let ast = match Parser::new(&src).parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("{}", e.to_string());
            process::exit(1);
        }
    };

    let mut evaluator = ASTEvaluator::new(argv);
    if let Err(e) = evaluator.eval(ast) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn interpret() {
    let mut evaluator = ASTEvaluator::new(vec![]);
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        let program = match Parser::new(&buffer).parse() {
            Ok(prog) => prog,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        let lines = match evaluator.eval(program) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        for option in lines {
            if let Some(value) = option {
                println!("{}", value);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 {
        parse_file(args);
    } else {
        interpret()
    }
}
