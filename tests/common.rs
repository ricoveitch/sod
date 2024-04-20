pub mod utils {
    use sod::ast::evaluator::ASTEvaluator;
    use sod::parser::Parser;
    use sod::symbol::symbol::Symbol;

    pub fn eval_expr(expr: &str) -> Vec<Option<Symbol>> {
        let mut evaluator = ASTEvaluator::new(vec![]);
        let program = Parser::new(expr).parse().unwrap();
        evaluator.eval(program).unwrap()
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
            let program = Parser::new(expr).parse().unwrap();
            let evaluation = evaluator.eval(program).unwrap();
            let symbol = evaluation.last().unwrap().as_ref().unwrap();
            assert_eq!(expected, symbol);
        }
    }
}
