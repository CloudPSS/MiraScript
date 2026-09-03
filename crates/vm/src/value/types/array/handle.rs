use super::*;

impl MiraArrayHandle {
    /// Return the number of elements in the array.
    pub fn len(self, runtime: &Runtime) -> Result<usize> {
        Ok(runtime.get_array_dyn(self)?.len())
    }

    /// Read one element by index of the array.
    pub fn get(self, runtime: &mut Runtime, index: usize) -> Result<MiraValue> {
        let result = runtime.get_array_dyn(self)?.get(self, runtime, index)?;
        runtime.insert(result)
    }
}
