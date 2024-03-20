use uuid::Uuid;

#[derive(PartialEq)]
pub enum ScopeKind {
    Global,
    FunctionBlock,
    ConditionalBlock,
}

pub struct Scope {
    pub name: String,
    kind: ScopeKind,
}

pub struct ScopeStack {
    scope: Vec<Vec<Scope>>,
}

impl ScopeStack {
    pub fn new() -> ScopeStack {
        ScopeStack {
            scope: vec![vec![Scope {
                kind: ScopeKind::Global,
                name: "global".to_string(),
            }]],
        }
    }

    pub fn curr_stack(&self) -> &Vec<Scope> {
        self.scope.last().unwrap()
    }

    pub fn curr(&self) -> &Scope {
        return self.curr_stack().last().unwrap();
    }

    fn push_scope_stack(&mut self, with: ScopeKind) -> String {
        let name = Uuid::new_v4().to_string();
        self.scope.push(vec![
            Scope {
                kind: ScopeKind::Global,
                name: "global".to_string(),
            },
            Scope {
                name: name.clone(),
                kind: with,
            },
        ]);
        name
    }

    fn push_scope(&mut self, with: ScopeKind) -> String {
        let name = Uuid::new_v4().to_string();
        match self.scope.last_mut() {
            Some(stack) => stack.push(Scope {
                name: name.clone(),
                kind: with,
            }),
            None => panic!("no scope found"),
        };
        name
    }

    pub fn push(&mut self, kind: ScopeKind) -> String {
        match kind {
            ScopeKind::ConditionalBlock => self.push_scope(kind),
            ScopeKind::FunctionBlock => self.push_scope_stack(kind),
            ScopeKind::Global => panic!("not able to push another global scope"),
        }
    }

    pub fn pop(&mut self) -> Scope {
        let popped_scope = match self.scope.last_mut().unwrap().pop() {
            Some(s) => s,
            None => panic!("scope out of bounds"),
        };

        if self.scope.len() > 1 && self.curr().kind == ScopeKind::Global {
            self.scope.pop();
        }

        popped_scope
    }
}
