use std::ops::DerefMut;

use crate::lexer::Token;

use super::{Expression, Pattern, Statement};

struct AstVisitorImpl<
    'a,
    T: FnMut(&mut Token<'a>),
    E: FnMut(&mut Expression<'a>),
    P: FnMut(&mut Pattern<'a>),
    S: FnMut(&mut Statement<'a>),
> {
    pub(crate) token: Option<T>,
    pub(crate) expression: Option<E>,
    pub(crate) pattern: Option<P>,
    pub(crate) statement: Option<S>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<
    'a,
    T: FnMut(&mut Token<'a>),
    E: FnMut(&mut Expression<'a>),
    P: FnMut(&mut Pattern<'a>),
    S: FnMut(&mut Statement<'a>),
> AstVisitor<'a> for AstVisitorImpl<'a, T, E, P, S>
{
    fn visit_token(&mut self, token: &mut Token<'a>) {
        if let Some(ref mut token_visitor) = self.token {
            token_visitor(token);
        }
    }
    fn visit_expression(&mut self, expression: &mut Expression<'a>) {
        if let Some(ref mut expression_visitor) = self.expression {
            expression_visitor(expression);
        }
    }
    fn visit_pattern(&mut self, pattern: &mut Pattern<'a>) {
        if let Some(ref mut pattern_visitor) = self.pattern {
            pattern_visitor(pattern);
        }
    }
    fn visit_statement(&mut self, statement: &mut Statement<'a>) {
        if let Some(ref mut statement_visitor) = self.statement {
            statement_visitor(statement);
        }
    }
}

pub(crate) trait AstVisitor<'a> {
    fn visit_token(&mut self, token: &mut Token<'a>) {
        token;
    }
    fn visit_expression(&mut self, expression: &mut Expression<'a>) {
        expression;
    }
    fn visit_pattern(&mut self, pattern: &mut Pattern<'a>) {
        pattern;
    }
    fn visit_statement(&mut self, statement: &mut Statement<'a>) {
        statement;
    }
}
pub(crate) fn walker<'a>(
    token: impl FnMut(&mut Token<'a>),
    expression: impl FnMut(&mut Expression<'a>),
    pattern: impl FnMut(&mut Pattern<'a>),
    statement: impl FnMut(&mut Statement<'a>),
) -> impl AstVisitor<'a> {
    AstVisitorImpl {
        token: Some(token),
        expression: Some(expression),
        pattern: Some(pattern),
        statement: Some(statement),
        _marker: std::marker::PhantomData,
    }
}

pub(crate) trait AstWalker<'a> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'a>);
}

impl<'a> AstWalker<'a> for Token<'a> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'a>) {
        visitor.visit_token(self);
    }
}

impl<'a, E: AstWalker<'a>> AstWalker<'a> for Vec<E> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'a>) {
        for item in self.iter_mut() {
            item.walk(visitor);
        }
    }
}

impl<'a, E: AstWalker<'a>> AstWalker<'a> for Option<E> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'a>) {
        if let Some(item) = self {
            item.walk(visitor);
        }
    }
}

impl<'a, E: AstWalker<'a>> AstWalker<'a> for Box<E> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'a>) {
        self.deref_mut().walk(visitor);
    }
}
