use super::MiraValue;
use crate::{Result, interpreter::Runtime};

/// A named collection of MiraScript-visible values.
pub trait MiraModule: std::any::Any + 'static {
    /// Return the module name shown in diagnostics and stack traces.
    fn name(&self) -> &str;

    /// Return the number of exported values in the module.
    fn len(&self) -> usize;

    /// Get index of a field by key, in iteration order, Returns [`None`] if the field does not exist.
    fn index_of(&self, key: &str) -> Option<usize>;

    /// Read a key by index, in iteration order, returning [`MiraError::MissingIndexOrField`] when the index is out of bounds.
    fn key(&self, index: usize) -> Result<&str>;

    /// Read a field by index, in iteration order, returning [`MiraError::MissingIndexOrField`] when the index is out of bounds.
    fn get(&self, runtime: &Runtime<'_>, index: usize) -> Result<MiraValue>;
}
