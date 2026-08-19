use super::MiraValue;
use crate::{Result, interpreter::Runtime};
use std::{fmt, rc::Rc};

const ANONYMOUS_FN_NAME: &str = "<anonymous>";

/// A native or execution-scoped script function.
pub trait MiraFunction: std::any::Any + 'static {
    /// Call this function with the given arguments, returning the result.
    fn call(&self, runtime: &Runtime<'_>, args: &[MiraValue]) -> Result<MiraValue>;

    /// Return the function name shown in diagnostics and stack traces.
    fn name(&self) -> &str {
        ANONYMOUS_FN_NAME
    }
}

type NativeCallback = dyn Fn(&Runtime<'_>, &[MiraValue]) -> Result<MiraValue>;

#[derive(Clone)]
/// A named, single-threaded native function callable from MiraScript.
pub struct MiraNativeFn {
    callback: Rc<NativeCallback>,
    name: Option<Rc<str>>,
}

fn wrap_callback<
    V: Into<MiraValue>,
    E: Into<anyhow::Error>,
    F: Fn(&Runtime<'_>, &[MiraValue]) -> std::result::Result<V, E> + 'static,
>(
    callback: F,
) -> impl Fn(&Runtime<'_>, &[MiraValue]) -> Result<MiraValue> + 'static {
    move |context, args| match callback(context, args) {
        Ok(value) => Ok(value.into()),
        Err(error) => Err(error.into().into()),
    }
}

impl MiraNativeFn {
    /// Create a native function with the given diagnostic name.
    ///
    /// # Examples
    ///
    /// ```
    /// use mirascript_vm::{MiraValue, MiraContext, MiraNativeFn, eval};
    /// use anyhow::bail;
    ///
    /// let mut context = MiraContext::empty();
    /// context.insert_fn("answer", MiraNativeFn::new("host.answer", |_, args| {
    ///     if args.len() != 0 {
    ///         bail!("expected 0 arguments");
    ///     }
    ///     Ok(42)
    /// }));
    /// assert_eq!(eval("answer()", &context)?, MiraAny::Number(42.0));
    /// # Ok::<(), Box<mirascript_vm::MiraError>>(())
    /// ```
    pub fn new<
        V: Into<MiraValue>,
        E: Into<anyhow::Error>,
        F: Fn(&Runtime<'_>, &[MiraValue]) -> std::result::Result<V, E> + 'static,
    >(
        name: impl Into<String>,
        callback: F,
    ) -> Self {
        MiraNativeFn {
            name: Some(Rc::from(name.into())),
            callback: Rc::new(wrap_callback(callback)),
        }
    }

    /// Create a native function named `<anonymous>`.
    pub fn anonymous<
        V: Into<MiraValue>,
        E: Into<anyhow::Error>,
        F: Fn(&Runtime<'_>, &[MiraValue]) -> std::result::Result<V, E> + 'static,
    >(
        callback: F,
    ) -> Self {
        MiraNativeFn {
            name: None,
            callback: Rc::new(wrap_callback(callback)),
        }
    }

    /// Create a native function from a callback that always succeeds.
    ///
    /// # Examples
    ///
    /// ```
    /// use mirascript_vm::{MiraValue, MiraContext, MiraNativeFn, eval};
    ///
    /// let mut context = MiraContext::empty();
    /// context.insert_fn("answer", MiraNativeFn::ok(|_, _| 42));
    /// assert_eq!(eval("answer()", &context)?, MiraValue::Number(42.0));
    /// # Ok::<(), Box<mirascript_vm::MiraError>>(())
    /// ```
    pub fn ok<V: Into<MiraValue>, F: Fn(&Runtime<'_>, &[MiraValue]) -> V + 'static>(
        callback: F,
    ) -> Self {
        Self {
            name: None,
            callback: Rc::new(move |context, args| Ok(callback(context, args).into())),
        }
    }

    /// Create a native function from a callback that always fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use mirascript_vm::{MiraValue, MiraContext, MiraNativeFn, MiraError, eval};
    /// use anyhow::anyhow;
    ///
    /// let mut context = MiraContext::empty();
    /// context.insert_fn("answer", MiraNativeFn::err(|_, _| anyhow!("no answer for you")));
    /// assert!(matches!(eval("answer()", &context).unwrap_err().as_ref(), MiraError::External { .. }));
    /// # Ok::<(), Box<mirascript_vm::MiraError>>(())
    /// ```
    pub fn err<E: Into<anyhow::Error>, F: Fn(&Runtime<'_>, &[MiraValue]) -> E + 'static>(
        callback: F,
    ) -> Self {
        Self {
            name: None,
            callback: Rc::new(move |context, args| Err(callback(context, args).into().into())),
        }
    }

    /// Create a native function from a callback that may fail with a [`MiraError`].
    pub fn builtin(
        name: impl Into<String>,
        callback: impl Fn(&Runtime<'_>, &[MiraValue]) -> Result<MiraValue> + 'static,
    ) -> Self {
        Self {
            name: Some(Rc::from(name.into())),
            callback: Rc::new(callback),
        }
    }

    /// Set the function name shown in diagnostics and stack traces.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(Rc::from(name.into()));
        self
    }

    pub(crate) fn shared_name(&self) -> Option<Rc<str>> {
        self.name.as_ref().map(Rc::clone)
    }

    pub(super) fn same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.callback, &other.callback)
    }
}

impl MiraFunction for MiraNativeFn {
    fn call(&self, runtime: &Runtime<'_>, args: &[MiraValue]) -> Result<MiraValue> {
        (self.callback)(runtime, args)
    }

    fn name(&self) -> &str {
        self.name.as_ref().map_or(ANONYMOUS_FN_NAME, AsRef::as_ref)
    }
}

impl fmt::Debug for MiraNativeFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MiraNativeFn").field(&self.name()).finish()
    }
}
