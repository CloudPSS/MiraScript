use std::{any::Any, fmt, rc::Rc};

use crate::{Result, Runtime, value::MiraManageable};

use super::MiraValue;

const ANONYMOUS_FN_NAME: &str = "<anonymous>";

/// A callable MiraScript function implementation.
pub trait MiraFunction: Any + 'static {
    /// Invoke the function.
    fn call(&self, runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraManageable>;

    /// Return the function name shown in diagnostics and stack traces.
    fn name(&self) -> &str {
        ANONYMOUS_FN_NAME
    }

    #[doc(hidden)]
    fn as_any(&self) -> &dyn Any;
}

type NativeCallback = dyn Fn(&mut Runtime, &[MiraValue]) -> Result<MiraManageable>;

/// A named, single-threaded native function callable from MiraScript.
#[derive(Clone)]
pub struct MiraNativeFn {
    callback: Rc<NativeCallback>,
    name: Option<Rc<str>>,
}

fn wrap_callback<V, E, F>(
    callback: F,
) -> impl Fn(&mut Runtime, &[MiraValue]) -> Result<MiraManageable> + 'static
where
    V: Into<MiraManageable>,
    E: Into<anyhow::Error>,
    F: Fn(&mut Runtime, &[MiraValue]) -> std::result::Result<V, E> + 'static,
{
    move |runtime, args| match callback(runtime, args) {
        Ok(value) => Ok(value.into()),
        Err(error) => Err(Box::<crate::MiraError>::from(error.into())),
    }
}

impl MiraNativeFn {
    /// Create a named native callback.
    pub fn new<V, E, F>(name: impl Into<String>, callback: F) -> Self
    where
        V: Into<MiraManageable>,
        E: Into<anyhow::Error>,
        F: Fn(&mut Runtime, &[MiraValue]) -> std::result::Result<V, E> + 'static,
    {
        Self {
            callback: Rc::new(wrap_callback(callback)),
            name: Some(Rc::from(name.into())),
        }
    }

    /// Create an anonymous native callback.
    pub fn anonymous<V, E, F>(callback: F) -> Self
    where
        V: Into<MiraManageable>,
        E: Into<anyhow::Error>,
        F: Fn(&mut Runtime, &[MiraValue]) -> std::result::Result<V, E> + 'static,
    {
        Self {
            callback: Rc::new(wrap_callback(callback)),
            name: None,
        }
    }

    /// Create an infallible anonymous callback.
    pub fn ok<V, F>(callback: F) -> Self
    where
        V: Into<MiraManageable>,
        F: Fn(&mut Runtime, &[MiraValue]) -> V + 'static,
    {
        Self {
            callback: Rc::new(move |runtime, args| Ok(callback(runtime, args).into())),
            name: None,
        }
    }

    /// Create a callback that always returns a host error.
    pub fn err<E, F>(callback: F) -> Self
    where
        E: Into<anyhow::Error>,
        F: Fn(&mut Runtime, &[MiraValue]) -> E + 'static,
    {
        Self {
            callback: Rc::new(move |runtime, args| {
                Err(Box::<crate::MiraError>::from(
                    callback(runtime, args).into(),
                ))
            }),
            name: None,
        }
    }

    /// Create an internal callback already using the VM result type.
    pub fn builtin(
        name: impl Into<String>,
        callback: impl Fn(&mut Runtime, &[MiraValue]) -> Result<MiraManageable> + 'static,
    ) -> Self {
        Self {
            callback: Rc::new(callback),
            name: Some(Rc::from(name.into())),
        }
    }

    /// Replace the diagnostic name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(Rc::from(name.into()));
        self
    }
}

impl MiraFunction for MiraNativeFn {
    fn call(&self, runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraManageable> {
        (self.callback)(runtime, args)
    }

    fn name(&self) -> &str {
        self.name.as_deref().unwrap_or(ANONYMOUS_FN_NAME)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl From<MiraNativeFn> for MiraManageable {
    fn from(value: MiraNativeFn) -> Self {
        Self::from_function(value)
    }
}

impl fmt::Debug for MiraNativeFn {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("MiraNativeFn")
            .field(&self.name())
            .finish()
    }
}
