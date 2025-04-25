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
pub struct Script<'a>(
    pub Vec<Statement<'a>>,
    pub Option<Box<Expression<'a>>>,
    pub Box<Token<'a>>,
);

struct ScriptIterMut<'a>(std::slice::Iter<'a, Statement<'a>>);

impl<'a> AstWalker<'a> for Script<'a> {
    fn walk(&mut self, visitor: &mut dyn AstVisitor<'a>) {
        for statement in &mut self.0 {
            statement.walk(visitor);
        }
        if let Some(expression) = &mut self.1 {
            expression.walk(visitor);
        }
        self.2.walk(visitor);
    }
}

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
            Self::write_ident(f, ident, "top ret")?;
            expression.fmt_ident(f, ident)?;
            writeln!(f)?;
        }
        Self::write_ident(f, ident, "EOF")?;
        self.2.fmt_ident(f, ident)?;
        Ok(())
    }
}
