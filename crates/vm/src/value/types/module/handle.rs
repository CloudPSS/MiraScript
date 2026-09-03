use super::*;

impl MiraHandle<dyn MiraModule> {
    /// Return the number of exports in the module.
    pub fn len(self, runtime: &Runtime) -> Result<usize> {
        Ok(runtime.get_module_dyn(self)?.len())
    }

    /// Find an export's iteration index of the module.
    pub fn index_of(self, runtime: &Runtime, key: &str) -> Result<Option<usize>> {
        Ok(runtime.get_module_dyn(self)?.index_of(key))
    }

    /// Read one export name by iteration index of the module.
    pub fn key(self, runtime: &Runtime, index: usize) -> Result<&str> {
        runtime.get_module_dyn(self)?.key(index)
    }

    /// Read one export value by iteration index of the module.
    pub fn get(self, runtime: &mut Runtime, index: usize) -> Result<MiraValue> {
        let result = runtime.get_module_dyn(self)?.get(self, runtime, index)?;
        runtime.insert(result)
    }
}
