use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

use super::{Expression, Statement};

#[derive(Debug, Clone, PartialEq)]
pub struct Script<'a>(pub Vec<Statement<'a>>, pub Option<Box<Expression<'a>>>);

impl<'a> Deref for Script<'a> {
    type Target = Vec<Statement<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Script<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for statement in self.iter() {
            write!(f, "{}", statement)?;
        }
        if let Some(expression) = &self.1 {
            write!(f, "{}", expression)?;
        }
        Ok(())
    }
}
