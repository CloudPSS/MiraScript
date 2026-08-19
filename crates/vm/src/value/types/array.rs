use crate::{MiraError, MiraValue, Result, interpreter::Runtime};

/// A MiraScript array.
pub trait MiraArray: std::any::Any + 'static {
    /// Return the number of elements in the array.
    fn len(&self) -> usize;

    /// Read an element by index, returning [`MiraError::MissingIndexOrField`] when the index is out of bounds.
    fn get(&self, runtime: &Runtime<'_>, index: usize) -> Result<MiraValue>;

    /// Return whether this array currently contains no elements.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: Into<MiraValue> + Clone + 'static> MiraArray for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }

    fn get(&self, _runtime: &Runtime<'_>, index: usize) -> Result<MiraValue> {
        let val = self
            .as_slice()
            .get(index)
            .ok_or_else(|| Box::new(MiraError::MissingIndexOrField))?;
        Ok(val.clone().into())
    }
}

impl<T: Into<MiraValue> + Clone + 'static, const N: usize> MiraArray for [T; N] {
    fn len(&self) -> usize {
        N
    }

    fn get(&self, _runtime: &Runtime<'_>, index: usize) -> Result<MiraValue> {
        if index >= N {
            return Err(Box::new(MiraError::MissingIndexOrField));
        }
        let val = &self[index];
        Ok(val.clone().into())
    }
}

impl<T: Into<MiraValue> + Clone + 'static> MiraArray for Box<[T]> {
    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn get(&self, _runtime: &Runtime<'_>, index: usize) -> Result<MiraValue> {
        let val = self
            .as_ref()
            .get(index)
            .ok_or_else(|| Box::new(MiraError::MissingIndexOrField))?;
        Ok(val.clone().into())
    }
}
