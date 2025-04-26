use crate::parser::Expression;

use super::{Closure, Variable};

pub(crate) struct Block<'s> {
    pub variables: Vec<Variable<'s>>,
    pub functions: Vec<Closure<'s>>,
    pub expression: Expression<'s>,
}
