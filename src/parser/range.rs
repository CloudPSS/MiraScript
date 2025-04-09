use std::fmt::Display;

use crate::ansi::DisplayIdent;

use super::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Range<'a> {
    /// `..`
    Closed(Box<Expression<'a>>, Box<Expression<'a>>),
    /// `..<`
    HalfOpen(Box<Expression<'a>>, Box<Expression<'a>>),
}

impl Display for Range<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl DisplayIdent for Range<'_> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        match self {
            Range::Closed(start, end) => {
                start.fmt_ident(f, ident)?;
                write!(f, "..")?;
                end.fmt_ident(f, ident)
            }
            Range::HalfOpen(start, end) => {
                start.fmt_ident(f, ident)?;
                write!(f, "..<")?;
                end.fmt_ident(f, ident)
            }
        }
    }
}
