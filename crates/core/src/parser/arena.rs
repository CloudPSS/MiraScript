use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StmtId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PatternId(pub usize);

#[derive(Debug, Clone, Default)]
pub struct ExprArena<'s> {
    items: Vec<Expression<'s>>,
}

impl<'s> ExprArena<'s> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    pub fn alloc(&mut self, value: Expression<'s>) -> ExprId {
        let id = ExprId(self.items.len());
        self.items.push(value);
        id
    }

    pub fn get(&self, id: ExprId) -> Option<&Expression<'s>> {
        self.items.get(id.0)
    }

    pub fn get_mut(&mut self, id: ExprId) -> Option<&mut Expression<'s>> {
        self.items.get_mut(id.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct StmtArena<'s> {
    items: Vec<Statement<'s>>,
}

impl<'s> StmtArena<'s> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    pub fn alloc(&mut self, value: Statement<'s>) -> StmtId {
        let id = StmtId(self.items.len());
        self.items.push(value);
        id
    }

    pub fn get(&self, id: StmtId) -> Option<&Statement<'s>> {
        self.items.get(id.0)
    }

    pub fn get_mut(&mut self, id: StmtId) -> Option<&mut Statement<'s>> {
        self.items.get_mut(id.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PatternArena<'s> {
    items: Vec<Pattern<'s>>,
}

impl<'s> PatternArena<'s> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    pub fn alloc(&mut self, value: Pattern<'s>) -> PatternId {
        let id = PatternId(self.items.len());
        self.items.push(value);
        id
    }

    pub fn get(&self, id: PatternId) -> Option<&Pattern<'s>> {
        self.items.get(id.0)
    }

    pub fn get_mut(&mut self, id: PatternId) -> Option<&mut Pattern<'s> {
        self.items.get_mut(id.0)
    }
}
