use std::borrow::Cow;

use super::opcode::Register;

/// Represents the type of binding for a variable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum BindType {
    /// The variable is bound by `let` statement.
    Let,
    /// The variable is bound as a pattern in `for`, `while`, `match` or `is` expressions.
    Init,
    /// The variable is bound as a function's normal parameter.
    Parameter,
    /// The variable is bound as a function's rest parameter.
    RestParameter,
    /// The variable is bound as a function statement.
    Func,
}

pub(crate) struct Variable<'s> {
    name: &'s str,
    mutable: bool,
    bind_type: BindType,
    register: Register,
    initialized: bool,
}

impl<'s> Variable<'s> {
    pub fn new(name: &'s str, mutable: bool, bind_type: BindType, register: Register) -> Self {
        Self {
            name,
            mutable,
            bind_type,
            register,
            initialized: matches!(bind_type, BindType::Parameter | BindType::RestParameter),
        }
    }

    pub fn initialize(&mut self) {
        self.initialized = true;
    }

    pub fn initialized(&self) -> bool {
        self.initialized
    }

    pub fn name(&self) -> &'s str {
        self.name
    }

    pub fn mutable(&self) -> bool {
        self.mutable
    }

    pub fn bind_type(&self) -> BindType {
        self.bind_type
    }

    pub fn register(&self) -> Register {
        self.register
    }
}
