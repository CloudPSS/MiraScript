use std::{
    collections::btree_map::IterMut,
    fmt::{self, Display, Formatter},
    ops::Deref,
};

use crate::{ansi::DisplayIdent, lexer::Token};

use super::{AstVisitor, AstWalker, Expression, Statement};

/// statement* expression? EOF
///
/// A script is a source file that contains a sequence of statements and an optional expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Script<'s>(
    pub Vec<Statement<'s>>,
    pub Option<Box<Expression<'s>>>,
    pub Box<Token<'s>>,
);

struct ScriptIterMut<'s>(std::slice::Iter<'s, Statement<'s>>);

impl<'s> AstWalker<'s> for Script<'s> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'s>) {
        for statement in &mut self.0 {
            statement.walk(visitor);
        }
        if let Some(expression) = &mut self.1 {
            expression.walk(visitor);
        }
        self.2.walk(visitor);
    }
}

impl<'s> Deref for Script<'s> {
    type Target = Vec<Statement<'s>>;

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
            Self::write_ident(f, ident, "top ret")?;
            expression.fmt_ident(f, ident)?;
            writeln!(f)?;
        }
        Self::write_ident(f, ident, "EOF")?;
        self.2.fmt_ident(f, ident)?;
        Ok(())
    }
}
