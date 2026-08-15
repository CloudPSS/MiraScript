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

type NativeCallback = dyn for<'a> Fn(&mut MiraCallContext<'a>, &[MiraAny]) -> Result<MiraAny>;

#[derive(Clone)]
/// A named, single-threaded native function callable from MiraScript.
pub struct MiraNativeFn {
    callback: Rc<NativeCallback>,
    name: Rc<str>,
}

impl MiraNativeFn {
    /// Create a native function with the given diagnostic name.
    ///
    /// # Examples
    ///
    /// ```
    /// use mira_vm::{MiraAny, MiraContext, MiraNativeFn, eval};
    ///
    /// let mut context = MiraContext::empty();
    /// context.insert_fn("answer", MiraNativeFn::new("host.answer", |_, _| {
    ///     Ok(MiraAny::Number(42.0))
    /// }));
    /// assert_eq!(eval("answer()", &context)?, MiraAny::Number(42.0));
    /// # Ok::<(), mira_vm::MiraError>(())
    /// ```
    pub fn new(
        name: impl Into<String>,
        callback: impl for<'a> Fn(&mut MiraCallContext<'a>, &[MiraAny]) -> Result<MiraAny> + 'static,
    ) -> Self {
        Self {
            callback: Rc::new(callback),
            name: Rc::from(name.into()),
        }
    }

    /// Create a native function named `<native>`.
    pub fn anonymous(
        callback: impl for<'a> Fn(&mut MiraCallContext<'a>, &[MiraAny]) -> Result<MiraAny> + 'static,
    ) -> Self {
        Self::new("<native>", callback)
    }

    /// Return the function name shown in diagnostics and stack traces.
    pub fn name(&self) -> &str {
        &self.name
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

impl<F> From<F> for MiraNativeFn
where
    F: for<'a> Fn(&mut MiraCallContext<'a>, &[MiraAny]) -> Result<MiraAny> + 'static,
{
    fn from(value: F) -> Self {
        Self::anonymous(value)
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
