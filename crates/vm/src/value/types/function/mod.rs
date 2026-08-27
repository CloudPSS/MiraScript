use std::any::Any;

use crate::{MiraHandle, Result, Runtime, value::MiraManageable};

use super::{MiraValue, MiraValueKind, value::ValueTag};

pub mod helper;
pub mod native;

pub use native::MiraNativeFn;

pub(crate) const ANONYMOUS_FN_NAME: &str = "<anonymous>";

impl MiraValue {
    /// Create a `MiraValue` representing a callable function.
    #[inline]
    pub const fn function<T: MiraFunction + ?Sized>(value: MiraHandle<T>) -> Self {
        Self::handle(ValueTag::Function, value.erase_function())
    }

    /// Check whether this value is a callable function.
    #[inline]
    pub const fn is_function(&self) -> bool {
        matches!(self.tag(), Some(ValueTag::Function))
    }

    /// Return the function handle, or `None` for another value type.
    #[inline]
    pub fn as_function(&self) -> Option<MiraHandle<dyn MiraFunction>> {
        match self.kind() {
            MiraValueKind::Function(value) => Some(value),
            _ => None,
        }
    }
}

/// A callable MiraScript function implementation.
pub trait MiraFunction: Any {
    /// Invoke the function.
    fn call(&self, runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraManageable>;

    /// Return the function name shown in diagnostics and stack traces.
    fn name(&self) -> &str {
        ANONYMOUS_FN_NAME
    }
}

impl<T> MiraFunction for T
where
    T: Fn(&mut Runtime, &[MiraValue]) -> Result<MiraManageable> + Any,
{
    fn call(&self, runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraManageable> {
        self(runtime, args)
    }
}
