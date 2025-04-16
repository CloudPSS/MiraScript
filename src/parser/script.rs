use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

use crate::{ansi::DisplayIdent, lexer::Token};

use super::{Expression, Statement};

/// statement* expression? EOF
///
/// A script is a source file that contains a sequence of statements and an optional expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Script<'a>(
    pub Vec<Statement<'a>>,
    pub Option<Box<Expression<'a>>>,
    pub Box<Token<'a>>,
);

impl<'a> Deref for Script<'a> {
    type Target = Vec<Statement<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Script<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_ident(f, 0)
    }
}

impl DisplayIdent for Script<'_> {
    fn fmt_ident(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        for statement in self.iter() {
            statement.fmt_ident(f, ident)?;
        }
        if let Some(expression) = &self.1 {
            Self::write_ident(f, ident, "return")?;
            expression.fmt_ident(f, ident)?;
            writeln!(f)?;
        }
        Self::write_ident(f, ident, "EOF")?;
        self.2.fmt_ident(f, ident)?;
        Ok(())
    }
}
