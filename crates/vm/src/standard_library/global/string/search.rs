use crate::standard_library::{insert_native, string};
use crate::{MiraValue, Runtime};

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "starts_with", |call, args| {
        let source = string(call, args, 0, "str")?;
        let search = string(call, args, 1, "search")?;
        Ok(MiraValue::boolean(source.starts_with(&search)))
    });
    insert_native(context, "ends_with", |call, args| {
        let source = string(call, args, 0, "str")?;
        let search = string(call, args, 1, "search")?;
        Ok(MiraValue::boolean(source.ends_with(&search)))
    });
    insert_native(context, "contains", |call, args| {
        let source = string(call, args, 0, "str")?;
        let search = string(call, args, 1, "search")?;
        Ok(MiraValue::boolean(source.contains(&search)))
    });
}
