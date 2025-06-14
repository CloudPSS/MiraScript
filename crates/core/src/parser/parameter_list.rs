use std::{
    fmt::{self, Display, Formatter},
    ops::{Deref, DerefMut},
};
use winnow::combinator::opt;

use crate::ansi::DisplayIdent;

use super::{
    ArrayElementBase, ArrayPattern, AstVisitor, AstWalker, patterns::array_pattern_like, prelude::*,
};

/// `(` ...items `)`
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterList<'s>(
    pub TokenRef<'s>,
    pub Vec<ArrayPattern<'s>>,
    pub TokenRef<'s>,
);

impl<'s> Deref for ParameterList<'s> {
    type Target = Vec<ArrayPattern<'s>>;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl DerefMut for ParameterList<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

impl<'s> AstWalker<'s> for ParameterList<'s> {
    fn collect_diagnostics(&mut self, collector: &mut Vec<SourceDiagnostic>) {
        self.0.collect_diagnostics(collector);
        self.1.collect_diagnostics(collector);
        self.2.collect_diagnostics(collector);
    }

    fn walk(&self, visitor: &mut dyn AstVisitor<'s>) {
        self.0.walk(visitor);
        self.1.walk(visitor);
        self.2.walk(visitor);
    }

    fn range(&self) -> SourceRange {
        self.0.range.start..self.2.range.end
    }
}

impl Display for ParameterList<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl DisplayIdent for ParameterList<'_> {
    fn fmt_ident(&self, f: &mut Formatter<'_>, indent: usize) -> fmt::Result {
        self.0.fmt_ident(f, indent)?;
        for item in self.1.iter() {
            item.fmt_ident(f, indent)?;
        }
        self.2.fmt_ident(f, indent)?;
        Ok(())
    }
}

pub(super) fn parameter_list<'s>(i: &mut Input<'s>) -> Result<Option<ParameterList<'s>>> {
    let list = opt(
        array_pattern_like([Operator::OpenParen, Operator::CloseParen], false).map(|pattern| {
            let Pattern::Array(left, items, right) = pattern else {
                unreachable!();
            };
            ParameterList(left, items, right)
        }),
    )
    .parse_next(i)?;
    let Some(mut list) = list else {
        return Ok(None);
    };
    let len = list.len();
    for (i, item) in list.iter_mut().enumerate() {
        let item = item.deref_mut();
        if i == len - 1 || !item.is_spread() {
            continue;
        }
        let ArrayElementBase::Spread(kw, p) = item else {
            unreachable!();
        };
        *item = ArrayElementBase::Element(Box::new(
            p.to_owned()
                .wrap_as_unknown([kw.clone()], DiagnosticCode::MispositionedRestParameter),
        ));
    }
    Ok(Some(list))
}
