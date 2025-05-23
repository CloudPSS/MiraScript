use crate::parser::Expression;

use super::variable::Variable;

pub(crate) struct Scope<'s> {
    pub variables: Vec<Variable<'s>>,
    pub level: usize,
}

impl<'s> Scope<'s> {
    pub fn new(level: usize) -> Self {
        Self {
            variables: Vec::new(),
            level,
        }
    }
    pub fn declare_variable(&mut self, variable: Variable<'s>) {
        self.variables.push(variable);
    }

    pub fn find_variable(&mut self, name: &str) -> Option<&mut Variable<'s>> {
        self.variables.iter_mut().find(|v| v.name() == name)
    }
}
