use std::{
    cmp::{max, min},
    ops::{Deref, DerefMut},
};

use crate::{diagnostic::SourceRange, lexer::Token};

use super::{Expression, Pattern, Statement};

struct AstVisitorMutImpl<
    's,
    T: FnMut(&mut Token<'s>),
    E: FnMut(&mut Expression<'s>),
    P: FnMut(&mut Pattern<'s>),
    S: FnMut(&mut Statement<'s>),
> {
    pub(crate) token: T,
    pub(crate) expression: E,
    pub(crate) pattern: P,
    pub(crate) statement: S,
    _marker: std::marker::PhantomData<&'s ()>,
}

impl<
    's,
    T: FnMut(&mut Token<'s>),
    E: FnMut(&mut Expression<'s>),
    P: FnMut(&mut Pattern<'s>),
    S: FnMut(&mut Statement<'s>),
> AstVisitorMut<'s> for AstVisitorMutImpl<'s, T, E, P, S>
{
    fn visit_token(&mut self, token: &mut Token<'s>) {
        (self.token)(token);
    }
    fn visit_expression(&mut self, expression: &mut Expression<'s>) {
        (self.expression)(expression);
    }
    fn visit_pattern(&mut self, pattern: &mut Pattern<'s>) {
        (self.pattern)(pattern);
    }
    fn visit_statement(&mut self, statement: &mut Statement<'s>) {
        (self.statement)(statement);
    }
}

pub(crate) trait AstVisitorMut<'s> {
    fn visit_token(&mut self, token: &mut Token<'s>) {
        token;
    }
    fn visit_expression(&mut self, expression: &mut Expression<'s>) {
        expression;
    }
    fn visit_pattern(&mut self, pattern: &mut Pattern<'s>) {
        pattern;
    }
    fn visit_statement(&mut self, statement: &mut Statement<'s>) {
        statement;
    }
}
pub(crate) fn walker_mut<'s>(
    token: impl FnMut(&mut Token<'s>),
    expression: impl FnMut(&mut Expression<'s>),
    pattern: impl FnMut(&mut Pattern<'s>),
    statement: impl FnMut(&mut Statement<'s>),
) -> impl AstVisitorMut<'s> {
    AstVisitorMutImpl {
        token,
        expression,
        pattern,
        statement,
        _marker: std::marker::PhantomData,
    }
}

struct AstVisitorImpl<
    's,
    T: FnMut(&Token<'s>),
    E: FnMut(&Expression<'s>),
    P: FnMut(&Pattern<'s>),
    S: FnMut(&Statement<'s>),
> {
    pub(crate) token: T,
    pub(crate) expression: E,
    pub(crate) pattern: P,
    pub(crate) statement: S,
    _marker: std::marker::PhantomData<&'s ()>,
}

impl<
    's,
    T: FnMut(&Token<'s>),
    E: FnMut(&Expression<'s>),
    P: FnMut(&Pattern<'s>),
    S: FnMut(&Statement<'s>),
> AstVisitor<'s> for AstVisitorImpl<'s, T, E, P, S>
{
    fn visit_token(&mut self, token: &Token<'s>) {
        (self.token)(token);
    }
    fn visit_expression(&mut self, expression: &Expression<'s>) {
        (self.expression)(expression);
    }
    fn visit_pattern(&mut self, pattern: &Pattern<'s>) {
        (self.pattern)(pattern);
    }
    fn visit_statement(&mut self, statement: &Statement<'s>) {
        (self.statement)(statement);
    }
}

pub(crate) trait AstVisitor<'s> {
    fn visit_token(&mut self, token: &Token<'s>) {
        token;
    }
    fn visit_expression(&mut self, expression: &Expression<'s>) {
        expression;
    }
    fn visit_pattern(&mut self, pattern: &Pattern<'s>) {
        pattern;
    }
    fn visit_statement(&mut self, statement: &Statement<'s>) {
        statement;
    }
}
pub(crate) fn walker<'s>(
    token: impl FnMut(&Token<'s>),
    expression: impl FnMut(&Expression<'s>),
    pattern: impl FnMut(&Pattern<'s>),
    statement: impl FnMut(&Statement<'s>),
) -> impl AstVisitor<'s> {
    AstVisitorImpl {
        token,
        expression,
        pattern,
        statement,
        _marker: std::marker::PhantomData,
    }
}

pub(crate) trait AstWalker<'s> {
    fn walk_mut(&mut self, visitor: &mut dyn AstVisitorMut<'s>);
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>);

    fn range(&self) -> SourceRange {
        let mut range = SourceRange {
            start: usize::MAX,
            end: usize::MIN,
        };
        self.walk(&mut walker(
            |token| {
                range.start = min(range.start, token.range.start);
                range.end = max(range.end, token.range.end);
            },
            |_| (),
            |_| (),
            |_| (),
        ));
        range
    }
}

impl<'s> AstWalker<'s> for Token<'s> {
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        visitor.visit_token(self);
    }
    fn walk_mut(&mut self, visitor: &mut dyn AstVisitorMut<'s>) {
        visitor.visit_token(self);
    }
}

impl<'s, E: AstWalker<'s>> AstWalker<'s> for Vec<E> {
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        for item in self.iter() {
            item.walk(visitor);
        }
    }
    fn walk_mut(&mut self, visitor: &mut dyn AstVisitorMut<'s>) {
        for item in self.iter_mut() {
            item.walk_mut(visitor);
        }
    }
}

impl<'s, E: AstWalker<'s>> AstWalker<'s> for Option<E> {
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        if let Some(item) = self {
            item.walk(visitor);
        }
    }
    fn walk_mut(&mut self, visitor: &mut dyn AstVisitorMut<'s>) {
        if let Some(item) = self {
            item.walk_mut(visitor);
        }
    }
}

impl<'s, E: AstWalker<'s>> AstWalker<'s> for Box<E> {
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        self.deref().walk(visitor);
    }
    fn walk_mut(&mut self, visitor: &mut dyn AstVisitorMut<'s>) {
        self.deref_mut().walk_mut(visitor);
    }
}
