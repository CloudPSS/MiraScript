use crate::Runtime;
use crate::standard_library::{insert_native, string};

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "to_uppercase", |call, args| {
        let value = string(call, args, 0, "str")?.to_uppercase();
        call.insert(value)
    });
    insert_native(context, "to_lowercase", |call, args| {
        let value = string(call, args, 0, "str")?.to_lowercase();
        call.insert(value)
    });
}
