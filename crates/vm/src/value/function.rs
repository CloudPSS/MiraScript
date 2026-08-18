use std::fmt;
use std::rc::Rc;

use crate::{Result, RunOptions};

use super::MiraAny;

pub(crate) trait NativeRuntime {
    fn call_value(&mut self, function: &MiraAny, args: &[MiraAny]) -> Result<MiraAny>;
    fn get_value(&self, value: &MiraAny, key: &MiraAny) -> Result<MiraAny>;
    fn options(&self) -> &RunOptions;
    fn checkpoint(&mut self) -> Result<()>;
}

/// Capabilities available to a native callback during one VM execution.
///
/// The context cannot outlive the callback invocation.
pub struct MiraCallContext<'a> {
    pub(crate) runtime: &'a mut dyn NativeRuntime,
}

impl MiraCallContext<'_> {
    /// Call a native, script, or callable extern value.
    pub fn call(&mut self, function: &MiraAny, args: &[MiraAny]) -> Result<MiraAny> {
        self.runtime.call_value(function, args)
    }

    /// Read a field or index using the same rules as a MiraScript expression.
    pub fn get(&mut self, value: &MiraAny, key: impl Into<MiraAny>) -> Result<MiraAny> {
        self.runtime.get_value(value, &key.into())
    }

    /// Return the options for the current execution.
    pub fn options(&self) -> &RunOptions {
        self.runtime.options()
    }

    /// Cooperatively check the current execution's timeout.
    ///
    /// Long-running native callbacks should call this method periodically.
    pub fn checkpoint(&mut self) -> Result<()> {
        self.runtime.checkpoint()
    }
}

type NativeCallback = dyn Fn(&mut MiraCallContext<'_>, &[MiraAny]) -> Result<MiraAny>;

#[derive(Clone)]
/// A named, single-threaded native function callable from MiraScript.
pub struct MiraNativeFn {
    callback: Rc<NativeCallback>,
    name: Rc<str>,
}

fn wrap_callback<
    V: Into<MiraAny>,
    E: Into<anyhow::Error>,
    F: Fn(&mut MiraCallContext<'_>, &[MiraAny]) -> std::result::Result<V, E> + 'static,
>(
    callback: F,
) -> impl Fn(&mut MiraCallContext<'_>, &[MiraAny]) -> Result<MiraAny> + 'static {
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
    /// use mirascript_vm::{MiraAny, MiraContext, MiraNativeFn, eval};
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
        V: Into<MiraAny>,
        E: Into<anyhow::Error>,
        F: Fn(&mut MiraCallContext<'_>, &[MiraAny]) -> std::result::Result<V, E> + 'static,
    >(
        name: impl Into<String>,
        callback: F,
    ) -> Self {
        MiraNativeFn {
            name: Rc::from(name.into()),
            callback: Rc::new(wrap_callback(callback)),
        }
    }

    /// Create a native function named `<anonymous>`.
    pub fn anonymous<
        V: Into<MiraAny>,
        E: Into<anyhow::Error>,
        F: Fn(&mut MiraCallContext<'_>, &[MiraAny]) -> std::result::Result<V, E> + 'static,
    >(
        callback: F,
    ) -> Self {
        Self::new("<anonymous>", callback)
    }

    /// Create a native function from a callback that always succeeds.
    ///
    /// # Examples
    ///
    /// ```
    /// use mirascript_vm::{MiraAny, MiraContext, MiraNativeFn, eval};
    ///
    /// let mut context = MiraContext::empty();
    /// context.insert_fn("answer", MiraNativeFn::ok(|_, _| 42));
    /// assert_eq!(eval("answer()", &context)?, MiraAny::Number(42.0));
    /// # Ok::<(), Box<mirascript_vm::MiraError>>(())
    /// ```
    pub fn ok<V: Into<MiraAny>, F: Fn(&mut MiraCallContext<'_>, &[MiraAny]) -> V + 'static>(
        callback: F,
    ) -> Self {
        Self {
            name: Rc::from("<anonymous>"),
            callback: Rc::new(move |context, args| Ok(callback(context, args).into())),
        }
    }

    /// Create a native function from a callback that always fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use mirascript_vm::{MiraAny, MiraContext, MiraNativeFn, MiraError, eval};
    /// use anyhow::anyhow;
    ///
    /// let mut context = MiraContext::empty();
    /// context.insert_fn("answer", MiraNativeFn::err(|_, _| anyhow!("no answer for you")));
    /// assert!(matches!(eval("answer()", &context).unwrap_err().as_ref(), MiraError::External { .. }));
    /// # Ok::<(), Box<mirascript_vm::MiraError>>(())
    /// ```
    pub fn err<
        E: Into<anyhow::Error>,
        F: Fn(&mut MiraCallContext<'_>, &[MiraAny]) -> E + 'static,
    >(
        callback: F,
    ) -> Self {
        Self {
            name: Rc::from("<anonymous>"),
            callback: Rc::new(move |context, args| Err(callback(context, args).into().into())),
        }
    }

    /// Create a native function from a callback that may fail with a [`MiraError`].
    pub fn builtin(
        name: impl Into<String>,
        callback: impl Fn(&mut MiraCallContext<'_>, &[MiraAny]) -> Result<MiraAny> + 'static,
    ) -> Self {
        Self {
            name: Rc::from(name.into()),
            callback: Rc::new(callback),
        }
    }

    /// Return the function name shown in diagnostics and stack traces.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the function name shown in diagnostics and stack traces.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Rc::from(name.into());
        self
    }

    pub(crate) fn shared_name(&self) -> Rc<str> {
        Rc::clone(&self.name)
    }

    pub(crate) fn call(
        &self,
        context: &mut MiraCallContext<'_>,
        args: &[MiraAny],
    ) -> Result<MiraAny> {
        (self.callback)(context, args)
    }

    pub(super) fn same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.callback, &other.callback)
    }
}

impl fmt::Debug for MiraNativeFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MiraNativeFn").field(&self.name).finish()
    }
}

#[derive(Debug, Clone)]
/// A native or execution-scoped script function.
pub enum MiraFunction {
    /// A host callback that may safely outlive an execution.
    Native(MiraNativeFn),
    #[doc(hidden)]
    Script {
        execution: u64,
        function: usize,
        frame: usize,
        name: Option<Rc<str>>,
    },
}

impl MiraFunction {
    /// Return the function's diagnostic name, when available.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Native(function) => Some(function.name()),
            Self::Script { name, .. } => name.as_deref(),
        }
    }

    pub(super) fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Native(a), Self::Native(b)) => a.same(b),
            (
                Self::Script {
                    execution: ae,
                    function: af,
                    frame: ac,
                    ..
                },
                Self::Script {
                    execution: be,
                    function: bf,
                    frame: bc,
                    ..
                },
            ) => ae == be && af == bf && ac == bc,
            _ => false,
        }
    }
}
