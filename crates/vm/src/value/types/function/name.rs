use std::{fmt::Display, rc::Rc};

/// A function name used for diagnostics and stack traces.
#[derive(Clone, Debug)]
pub enum FunctionName {
    /// A static function name known at compile time.
    Static(&'static str),
    /// A dynamic function name known only at runtime.
    Dynamic(Rc<str>),
}

impl Display for FunctionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionName::Static(name) => write!(f, "{}", name),
            FunctionName::Dynamic(name) => write!(f, "{}", name),
        }
    }
}

impl AsRef<str> for FunctionName {
    fn as_ref(&self) -> &str {
        match self {
            FunctionName::Static(name) => name,
            FunctionName::Dynamic(name) => name.as_ref(),
        }
    }
}

impl PartialEq for FunctionName {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl PartialEq<&str> for FunctionName {
    fn eq(&self, other: &&str) -> bool {
        self.as_ref() == *other
    }
}

const ANONYMOUS_FN_NAME: &str = "<anonymous>";

impl FunctionName {
    /// Create a function name from a static string.
    pub const fn static_name(name: &'static str) -> Self {
        Self::Static(name)
    }

    /// Create a function name from a dynamic string.
    pub fn dynamic_name(name: impl Into<Rc<str>>) -> Self {
        Self::Dynamic(name.into())
    }

    /// Default function name for anonymous functions.
    pub const fn anonymous() -> Self {
        Self::Static(ANONYMOUS_FN_NAME)
    }
}

impl From<&'static str> for FunctionName {
    fn from(name: &'static str) -> Self {
        Self::Static(name)
    }
}

impl From<String> for FunctionName {
    fn from(name: String) -> Self {
        Self::Dynamic(Rc::from(name))
    }
}
