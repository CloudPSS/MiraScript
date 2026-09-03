use super::*;

impl MiraHandle<dyn MiraRecord> {
    /// Return the number of fields in the record.
    pub fn len(self, runtime: &Runtime) -> Result<usize> {
        Ok(runtime.get_record_dyn(self)?.len())
    }

    /// Find a field's iteration index of the record.
    pub fn index_of(self, runtime: &Runtime, key: &str) -> Result<Option<usize>> {
        Ok(runtime.get_record_dyn(self)?.index_of(key))
    }

    /// Find an integer field key's iteration index of the record without allocating when possible.
    pub fn index_of_i(self, runtime: &Runtime, key: u32) -> Result<Option<usize>> {
        Ok(runtime.get_record_dyn(self)?.index_of_i(key))
    }

    /// Read one field key by iteration index of the record.
    pub fn key(self, runtime: &Runtime, index: usize) -> Result<&str> {
        runtime.get_record_dyn(self)?.key(index)
    }

    /// Read one field by iteration index of the record.
    pub fn get(self, runtime: &mut Runtime, index: usize) -> Result<MiraValue> {
        let result = runtime.get_record_dyn(self)?.get(self, runtime, index)?;
        runtime.insert(result)
    }
}
