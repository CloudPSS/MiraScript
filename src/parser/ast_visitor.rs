use std::ops::DerefMut;

use crate::lexer::Token;

use super::{Expression, Pattern, Statement};

struct AstVisitorImpl<
    's,
    T: FnMut(&mut Token<'s>),
    E: FnMut(&mut Expression<'s>),
    P: FnMut(&mut Pattern<'s>),
    S: FnMut(&mut Statement<'s>),
> {
    pub(crate) token: Option<T>,
    pub(crate) expression: Option<E>,
    pub(crate) pattern: Option<P>,
    pub(crate) statement: Option<S>,
    _marker: std::marker::PhantomData<&'s ()>,
}

impl<
    's,
    T: FnMut(&mut Token<'s>),
    E: FnMut(&mut Expression<'s>),
    P: FnMut(&mut Pattern<'s>),
    S: FnMut(&mut Statement<'s>),
> AstVisitor<'s> for AstVisitorImpl<'s, T, E, P, S>
{
    fn visit_token(&mut self, token: &mut Token<'s>) {
        if let Some(ref mut token_visitor) = self.token {
            token_visitor(token);
        }
    }
    fn visit_expression(&mut self, expression: &mut Expression<'s>) {
        if let Some(ref mut expression_visitor) = self.expression {
            expression_visitor(expression);
        }
    }
    fn visit_pattern(&mut self, pattern: &mut Pattern<'s>) {
        if let Some(ref mut pattern_visitor) = self.pattern {
            pattern_visitor(pattern);
        }
    }
    fn visit_statement(&mut self, statement: &mut Statement<'s>) {
        if let Some(ref mut statement_visitor) = self.statement {
            statement_visitor(statement);
        }
    }
}

pub(crate) trait AstVisitor<'s> {
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
pub(crate) fn walker<'s>(
    token: impl FnMut(&mut Token<'s>),
    expression: impl FnMut(&mut Expression<'s>),
    pattern: impl FnMut(&mut Pattern<'s>),
    statement: impl FnMut(&mut Statement<'s>),
) -> impl AstVisitor<'s> {
    AstVisitorImpl {
        token: Some(token),
        expression: Some(expression),
        pattern: Some(pattern),
        statement: Some(statement),
        _marker: std::marker::PhantomData,
    }
}

pub(crate) trait AstWalker<'s> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'s>);
}

impl<'s> AstWalker<'s> for Token<'s> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'s>) {
        visitor.visit_token(self);
    }
}

impl<'s, E: AstWalker<'s>> AstWalker<'s> for Vec<E> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'s>) {
        for item in self.iter_mut() {
            item.walk(visitor);
        }
    }
}

impl<'s, E: AstWalker<'s>> AstWalker<'s> for Option<E> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'s>) {
        if let Some(item) = self {
            item.walk(visitor);
        }
    }
}

impl<'s, E: AstWalker<'s>> AstWalker<'s> for Box<E> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'s>) {
        self.deref_mut().walk(visitor);
    }
}
