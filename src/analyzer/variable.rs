use std::borrow::Cow;

/// Represents the type of binding for a variable.
pub(crate) enum BindType {
    /// The variable is bound by `let` statement.
    Scope,
    /// The variable is bound as a pattern in `for`, `if`, `while`, or `match` expressions.
    ScopeInit,
    /// The variable is bound as a function's normal parameter.
    Parameter,
    /// The variable is bound as a function's rest parameter.
    RestParameter,
}

pub(crate) struct Variable<'a> {
    pub name: Cow<'a, str>,
    pub is_mutable: bool,
    pub bind_type: BindType,
}
