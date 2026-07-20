use std::ops::{Deref, DerefMut};

use winnow::combinator::opt;

use super::{
    helper::{token, token_or_insert},
    patterns::array_pattern_like,
    prelude::*,
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
    fn collect_diagnostics(&mut self, collector: &mut DiagnosticsCollector<'_, '_>) {
        self.0.collect_diagnostics(collector);
        self.1.collect_diagnostics(collector);
        self.2.collect_diagnostics(collector);
    }
    fn range(&self) -> SourceRange {
        self.0.range.start..self.2.range.end
    }
}

pub(super) fn parameter_list<'s>(i: &mut Input<'s>) -> Result<Option<ParameterList<'s>>> {
    let list = opt(array_pattern_like(
        token(Operator::OpenParen),
        token_or_insert(Operator::CloseParen, DiagnosticCode::MissingCloseParen),
        false,
    )
    .map(|pattern| {
        let Pattern::Array(left, items, right) = pattern else {
            unreachable!();
        };
        ParameterList(left, items, right)
    }))
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
        let pattern = std::mem::replace(&mut **p, Pattern::SpreadDiscard(kw.range.start));
        *item = ArrayElementBase::Element(Box::new(
            pattern.wrap_as_unknown([kw.clone()], DiagnosticCode::MispositionedRestParameter),
        ));
    }
    Ok(Some(list))
}
