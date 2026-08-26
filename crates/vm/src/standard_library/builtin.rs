use crate::{MiraFunction, MiraManageable, MiraValue, Result, Runtime};

struct BuiltinFn<F: Fn(&mut Runtime, &[MiraValue]) -> Result<MiraValue> + 'static> {
    name: &'static str,
    callback: F,
}

impl<F: Fn(&mut Runtime, &[MiraValue]) -> Result<MiraValue>> MiraFunction for BuiltinFn<F> {
    fn call(&self, runtime: &mut Runtime, args: &[MiraValue]) -> Result<crate::MiraManageable> {
        (self.callback)(runtime, args).map(Into::into)
    }

    fn name(&self) -> &str {
        self.name
    }
}

pub(crate) fn builtin_fn(
    name: &'static str,
    callback: impl Fn(&mut Runtime, &[MiraValue]) -> Result<MiraValue> + 'static,
) -> MiraManageable {
    MiraManageable::from_function(BuiltinFn { name, callback })
}

/// A macro to define a global builtin function and insert it into the runtime.
#[macro_export]
macro_rules! global_builtin (
    ($runtime:ident, $($tokens:tt)*) => {
        global_builtin!(@item $runtime, $($tokens)*);
    };

    (@item $runtime:ident, ) => {};

    // 直接定义函数
    (@item $runtime:ident, fn $id:ident ($call:ident, $args:ident) $body:block) => {{
        let name: &'static str = stringify!($id);
        let full_name: &'static str = concat!("global.", stringify!($id));

        #[allow(nonstandard_style)]
        fn $id(
            $call: &mut $crate::Runtime,
            $args: &[$crate::MiraValue],
        ) -> $crate::Result<$crate::MiraValue>
        $body

        let f = $crate::standard_library::builtin_fn(full_name, $id);
        $runtime.insert_std(name, f);
    }};
    (@item $runtime:ident, fn $id:ident ($call:ident, $args:ident) $body:block $($rest:tt)*) => {{
        global_builtin!(@item $runtime, fn $id($call, $args) $body);
        global_builtin!(@item $runtime, $($rest)*);
    }};

    // 引用已有实现
    (@item $runtime:ident, fn $id:ident : $impl:expr) => {{
        let name: &'static str = stringify!($id);
        let full_name: &'static str = concat!("global.", stringify!($id));

        #[allow(nonstandard_style)]
        fn $id(
            call: &mut $crate::Runtime,
            args: &[$crate::MiraValue],
        ) -> $crate::Result<$crate::MiraValue> {
            $impl(call, args)
        }

        let f = $crate::standard_library::builtin_fn(full_name, $id);
        $runtime.insert_std(name, f);
    }};
    (@item $runtime:ident, fn $id:ident : $impl:expr ; $($rest:tt)*) => {{
        global_builtin!(@item $runtime, fn $id : $impl);
        global_builtin!(@item $runtime, $($rest)*);
    }};

    // 定义常量
    (@item $runtime:ident, let $id:ident = $value:expr) => {{
        let name: &'static str = stringify!($id);
        #[allow(nonstandard_style)]
        let $id = $value;
        $runtime.insert_std(name, $crate::MiraValue::from($id));
    }};
    (@item $runtime:ident, let $id:ident = $value:expr ; $($rest:tt)*) => {{
        global_builtin!(@item $runtime, let $id = $value);
        global_builtin!(@item $runtime, $($rest)*);
    }};
);

pub(crate) use global_builtin;
