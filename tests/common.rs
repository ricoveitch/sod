pub mod utils {
    use orca::ast::evaluator::ASTEvaluator;
    use orca::parser::Parser;
    use orca::symbol::symbol::Symbol;

    pub fn eval_expr(expr: &str) -> Vec<Option<Symbol>> {
        let mut evaluator = ASTEvaluator::new(vec![]);
        let program = Parser::new(expr).parse();
        evaluator.eval(program)
    }

    pub fn assert_expr(expr: &str, expected: Symbol) {
        let evaluation = eval_expr(expr);
        let symbol = evaluation.last().unwrap().as_ref().unwrap();
        assert_eq!(&expected, symbol);
    }

    #[allow(dead_code)]
    pub fn assert_exprs(exprs: Vec<&str>, expected: Vec<Symbol>) {
        let mut evaluator = ASTEvaluator::new(vec![]);
        for (expr, expected) in exprs.iter().zip(expected.iter()) {
            let program = Parser::new(expr).parse();
            let evaluation = evaluator.eval(program);
            let symbol = evaluation.last().unwrap().as_ref().unwrap();
            assert_eq!(expected, symbol);
        }
    }
}
