#[derive(PartialEq)]
pub enum ScopeKind {
    Global,
    FunctionBlock,
    ConditionalBlock,
}

pub struct Scope {
    pub id: usize,
    kind: ScopeKind,
}
pub const GLOBAL_SCOPE_ID: usize = 0;

pub struct ScopeStack {
    scope: Vec<Vec<Scope>>,
    counter: usize,
}

impl ScopeStack {
    pub fn new() -> ScopeStack {
        ScopeStack {
            scope: vec![vec![Scope {
                kind: ScopeKind::Global,
                id: GLOBAL_SCOPE_ID,
            }]],
            counter: GLOBAL_SCOPE_ID + 1,
        }
    }

    pub fn curr_stack(&self) -> &Vec<Scope> {
        self.scope.last().unwrap()
    }

    pub fn curr(&self) -> &Scope {
        return self.curr_stack().last().unwrap();
    }

    fn push_scope_stack(&mut self, with: ScopeKind) -> usize {
        let id = self.counter;
        self.scope.push(vec![
            Scope {
                kind: ScopeKind::Global,
                id: GLOBAL_SCOPE_ID,
            },
            Scope { id, kind: with },
        ]);
        id
    }

    fn push_scope(&mut self, with: ScopeKind) -> usize {
        let id = self.counter;
        match self.scope.last_mut() {
            Some(stack) => stack.push(Scope { id, kind: with }),
            None => panic!("no scope found"),
        };
        id
    }

    pub fn push(&mut self, kind: ScopeKind) -> usize {
        let id = match kind {
            ScopeKind::ConditionalBlock => self.push_scope(kind),
            ScopeKind::FunctionBlock => self.push_scope_stack(kind),
            ScopeKind::Global => panic!("not able to push another global scope"),
        };

        self.counter += 1;
        id
    }

    pub fn pop(&mut self) -> Scope {
        let popped_scope = match self.scope.last_mut().unwrap().pop() {
            Some(s) => s,
            None => panic!("scope out of bounds"),
        };

        if self.scope.len() > 1 && self.curr().kind == ScopeKind::Global {
            self.scope.pop();
        }

        self.counter -= 1;
        popped_scope
    }
}
