use crate::standard_library::insert_native;
use crate::{MiraAny, MiraContext, MiraError, operations};

pub(super) fn install(context: &mut MiraContext) {
    insert_native(context, "debug_print", |call, args| {
        let message = args
            .iter()
            .map(operations::display)
            .collect::<Vec<_>>()
            .join(" ");
        call.options().providers.debug(&message);
        Ok(MiraAny::Nil)
    });
    insert_native(context, "panic", |_, args| {
        let message = args
            .first()
            .map(operations::to_string)
            .transpose()?
            .unwrap_or_else(|| "MiraScript panic".into());
        Err(MiraError::runtime(message))
    });
}
