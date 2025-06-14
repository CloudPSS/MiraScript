use std::{
    cmp::{max, min},
    ops::{Deref, DerefMut},
};

use super::prelude::*;

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
        _ = token;
    }
    fn visit_expression(&mut self, expression: &Expression<'s>) {
        _ = expression;
    }
    fn visit_pattern(&mut self, pattern: &Pattern<'s>) {
        _ = pattern;
    }
    fn visit_statement(&mut self, statement: &Statement<'s>) {
        _ = statement;
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
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>);

    fn walk(&self, visitor: &mut dyn AstVisitor<'s>);

    fn range(&self) -> SourceRange {
        self.range_slow()
    }

    fn range_slow(&self) -> SourceRange {
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
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        // Parsing 阶段不会创建新的 TokenKind::InterpolatedString，无需处理

        if !self.is_unknown() {
            return;
        }
        let TokenKind::Unknown {
            mut errors,
            recovered,
        } = std::mem::replace(&mut self.kind, TokenKind::Eof)
        else {
            unreachable!();
        };
        collector.append(&mut errors);
        if let Some(recovered) = recovered {
            self.kind = *recovered;
        } else {
            self.kind = TokenKind::Unknown {
                recovered: None,
                errors,
            }
        }
    }
    fn range_slow(&self) -> SourceRange {
        self.range.clone()
    }
}

impl<'s, E: AstWalker<'s>> AstWalker<'s> for Vec<E> {
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        for item in self.iter() {
            item.walk(visitor);
        }
    }
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        for item in self.iter_mut() {
            item.collect_diagnostics(collector);
        }
    }
    fn range(&self) -> SourceRange {
        if self.is_empty() {
            SourceRange {
                start: usize::MAX,
                end: usize::MIN,
            }
        } else {
            let first = self.first().unwrap().range();
            let last = self.last().unwrap().range();
            first.start..last.end
        }
    }
}

impl<'s, E: AstWalker<'s>> AstWalker<'s> for Option<E> {
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        if let Some(item) = self {
            item.walk(visitor);
        }
    }
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        if let Some(item) = self {
            item.collect_diagnostics(collector);
        }
    }
    fn range(&self) -> SourceRange {
        if let Some(item) = self {
            item.range()
        } else {
            SourceRange {
                start: usize::MAX,
                end: usize::MIN,
            }
        }
    }
    fn range_slow(&self) -> SourceRange {
        if let Some(item) = self {
            item.range_slow()
        } else {
            SourceRange {
                start: usize::MAX,
                end: usize::MIN,
            }
        }
    }
}

impl<'s, E: AstWalker<'s>> AstWalker<'s> for Box<E> {
    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        self.deref().walk(visitor);
    }
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        self.deref_mut().collect_diagnostics(collector);
    }
    fn range(&self) -> SourceRange {
        self.deref().range()
    }
    fn range_slow(&self) -> SourceRange {
        self.deref().range_slow()
    }
}
