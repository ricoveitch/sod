pub mod utils {
    use orca::ast::evaluator::ASTEvaluator;
    use orca::ast::symbol::Symbol;
    use orca::parser::Parser;

    pub fn eval_expr(expr: &str) -> Vec<Option<Symbol>> {
        let mut evaluator = ASTEvaluator::new();
        let program = Parser::new(expr).parse();
        evaluator.eval(program)
    }

    pub fn assert_expr(expr: &str, expected: Symbol) {
        let evaluation = eval_expr(expr);
        let symbol = evaluation.last().unwrap().as_ref().unwrap();
        assert_eq!(&expected, symbol);
    }
}
