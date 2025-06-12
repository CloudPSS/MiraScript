use std::{
    fmt::{self, Display, Formatter},
    ops::{Deref, DerefMut},
};
use winnow::{ModalResult, Parser, combinator::opt};

use crate::{
    ansi::DisplayIdent,
    diagnostic::SourceRange,
    lexer::{Operator, Token},
};

use super::{
    ArrayPattern, AstVisitor, AstVisitorMut, AstWalker, Input, Pattern,
    patterns::array_pattern_like,
};

/// `(` ...items `)`
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParameterList<'s>(
    pub Box<Token<'s>>,
    pub Vec<ArrayPattern<'s>>,
    pub Box<Token<'s>>,
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
    fn walk_mut(&mut self, visitor: &mut dyn AstVisitorMut<'s>) {
        self.0.walk_mut(visitor);
        self.1.walk_mut(visitor);
        self.2.walk_mut(visitor);
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

pub(super) fn parameter_list<'s>(i: &mut Input<'_, 's>) -> ModalResult<Option<ParameterList<'s>>> {
    opt(
        array_pattern_like([Operator::OpenParen, Operator::CloseParen], false).map(|pattern| {
            let Pattern::Array(left, items, right) = pattern else {
                unreachable!();
            };
            ParameterList(left, items, right)
        }),
    )
    .parse_next(i)
}

// pub(super) fn parameter_list<'s>(i: &mut Input<'_, 's>) -> ModalResult<Option<Vec<Token<'s>>>> {
//     let t = peek(any).parse_next(i)?;
//     if *t != Operator::OpenParen {
//         return Ok(None);
//     }

//     delimited(
//         token(Operator::OpenParen),
//         (
//             repeat(
//                 0..,
//                 terminated(
//                     one_of(|t: &Token<'s>| matches!(&t.kind, &TokenKind::Identifier(_))),
//                     token(Operator::Comma),
//                 ),
//             )
//             .fold(Vec::new, |mut v, t: &Token<'s>| {
//                 v.push(t.to_owned());
//                 v
//             }),
//             opt(one_of(|t: &Token<'s>| {
//                 matches!(&t.kind, &TokenKind::Identifier(_))
//             })),
//         ),
//         token(Operator::CloseParen),
//     )
//     .map(|(mut v, t): (Vec<Token<'s>>, Option<&Token<'s>>)| {
//         if let Some(t) = t {
//             v.push(t.to_owned());
//         }
//         Some(v)
//     })
//     .parse_next(i)
// }
