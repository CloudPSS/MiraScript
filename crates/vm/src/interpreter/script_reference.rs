use std::{any::Any, collections::HashSet};

use crate::{MiraHandle, MiraModule, MiraValue, MiraValueKind, Result};

use super::Runtime;

impl Runtime {
    /// Checks if the given value contains a reference to a script function or module.
    pub(crate) fn contains_script_reference(&mut self, value: MiraValue) -> Result<bool> {
        self.contains_script_reference_inner(value, &mut HashSet::new())
    }

    fn contains_script_reference_inner(
        &mut self,
        value: MiraValue,
        visited: &mut HashSet<MiraHandle<dyn MiraModule>>,
    ) -> Result<bool> {
        match value.kind() {
            MiraValueKind::Function(handle) => Ok(<dyn Any>::is::<super::ScriptFunction>(
                self.get_function_dyn(handle)?.as_ref(),
            )),
            MiraValueKind::Module(handle) => {
                if <dyn Any>::is::<super::ScriptModule>(self.get_module_dyn(handle)?) {
                    return Ok(true);
                }
                if !visited.insert(handle) {
                    return Ok(false);
                }
                for key in crate::operations::module_keys(self, value)?.unwrap_or_default() {
                    if let Some(item) = crate::operations::module_get(self, value, &key)?
                        && self.contains_script_reference_inner(item, visited)?
                    {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }
}
