use crate::parser::Expression;

use super::{Closure, Variable};

pub(crate) struct Block<'a> {
    pub variables: Vec<Variable<'a>>,
    pub functions: Vec<Closure<'a>>,
    pub expression: Expression<'a>,
}
