use crate::standard_library::{global_builtin, string};
use crate::{MiraValue, Runtime};

pub(super) fn install(context: &mut Runtime) {
    global_builtin!(context, fn starts_with(call, args) {
        let source = string(call, args, 0, "str")?;
        let search = string(call, args, 1, "search")?;
        Ok(MiraValue::boolean(source.starts_with(&search)))
    });
    global_builtin!(context, fn ends_with(call, args) {
        let source = string(call, args, 0, "str")?;
        let search = string(call, args, 1, "search")?;
        Ok(MiraValue::boolean(source.ends_with(&search)))
    });
    global_builtin!(context, fn contains(call, args) {
        let source = string(call, args, 0, "str")?;
        let search = string(call, args, 1, "search")?;
        Ok(MiraValue::boolean(source.contains(&search)))
    });
}
