use crate::standard_library::insert_native;
use crate::{MiraError, MiraValue, Runtime, RuntimeErrorKind, operations};

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "debug_print", |call, args| {
        let message = args
            .iter()
            .map(|value| operations::display(call, *value))
            .collect::<Vec<_>>()
            .join(" ");
        call.options().providers.debug(&message);
        Ok(MiraValue::Nil)
    });
    insert_native(context, "panic", |call, args| {
        let message = args
            .first()
            .map(|value| operations::to_string(call, *value))
            .transpose()?
            .unwrap_or_else(|| "MiraScript panic".into());
        Err(MiraError::runtime(RuntimeErrorKind::UserMessage {
            message,
        }))
    });
}
