use super::*;

impl MiraHandle<dyn MiraFunction> {
    /// Invoke the function.
    pub fn call(self, runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        runtime.call_function(self, args)
    }
}
