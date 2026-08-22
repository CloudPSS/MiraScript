use crate::standard_library::insert_native;
use crate::{MiraError, MiraValue, Runtime, RuntimeErrorKind, operations};

pub(super) fn install(context: &mut Runtime) {
    insert_native(context, "to_string", |call, args| {
        let value = args.first().cloned().ok_or_else(|| {
            MiraError::runtime(RuntimeErrorKind::MissingArgument { name: "data" })
        })?;
        let value = operations::to_string(call, value)?;
        call.insert(value)
    });
    insert_native(context, "to_number", |call, args| {
        let value = args.first().cloned().ok_or_else(|| {
            MiraError::runtime(RuntimeErrorKind::MissingArgument { name: "data" })
        })?;
        match operations::to_number(call, value) {
            Ok(value) => Ok(MiraValue::number(value)),
            Err(_) if args.len() > 1 => Ok(args[1]),
            Err(error) => Err(error),
        }
    });
    insert_native(context, "format", |call, args| {
        let value = args.first().cloned().ok_or_else(|| {
            MiraError::runtime(RuntimeErrorKind::MissingArgument { name: "data" })
        })?;
        if args.len() < 2 {
            return Err(MiraError::runtime(RuntimeErrorKind::MissingArgument {
                name: "format",
            }));
        }
        let specifier = match args.get(1) {
            Some(value) if value.is_nil() => None,
            Some(value) => Some(operations::to_string(call, *value)?),
            None => unreachable!(),
        };
        let value = operations::format_value(call, value, specifier.as_deref())?;
        call.insert(value)
    });
}
