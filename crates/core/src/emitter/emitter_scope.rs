use std::ops::{Deref, DerefMut};

use crate::{
    error::{ErrorCode, SourceError},
    lexer::Operator,
    parser::{Expression, Script, Statement},
};

use super::{
    Emitter,
    scope::Scope,
    variable::{BindType, Variable},
};

pub(super) struct Scopes<'s>(Vec<Scope<'s>>);

impl<'s> Scopes<'s> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn current(&mut self) -> &mut Scope<'s> {
        self.last_mut().unwrap()
    }

    pub fn find_variable(&mut self, name: &str) -> Option<(usize, &mut Variable<'s>)> {
        for scope in self.0.iter_mut().rev() {
            let level = scope.level;
            if let Some(var) = scope.find_variable(name) {
                return Some((level, var));
            }
        }
        None
    }

    pub fn find_local_variable(&mut self, name: &str) -> Option<&mut Variable<'s>> {
        self.current().find_variable(name)
    }

    pub fn check_local_variable(&mut self, name: &str) -> Option<ErrorCode> {
        let var = self.find_local_variable(name);
        var.map(|var| {
            if matches!(
                var.bind_type(),
                BindType::Parameter | BindType::RestParameter
            ) {
                ErrorCode::DuplicateParameterDeclaration
            } else {
                ErrorCode::DuplicateVariableDeclaration
            }
        })
    }
}

impl<'s> Deref for Scopes<'s> {
    type Target = Vec<Scope<'s>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Scopes<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'s> Emitter<'s> {
    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new(self.closures.len()));
    }
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }
    pub fn declare_variable(&mut self, name: &'s str, mutable: bool, bind_type: BindType) {
        let register = match bind_type {
            BindType::Parameter => self.current_closure().add_arg(),
            BindType::RestParameter => self.current_closure().add_var_arg(),
            _ => self.current_closure().add_reg(),
        };
        let var = Variable::new(name, mutable, bind_type, register);
        self.scopes.current().declare_variable(var);
    }
}
