use std::{borrow::Cow, fmt, rc::Rc};

use crate::{Result, Runtime, value::MiraManageable};

use super::{ANONYMOUS_FN_NAME, MiraFunction, MiraValue};

type NativeCallback = dyn Fn(&mut Runtime, &[MiraValue]) -> Result<MiraManageable>;

/// A named, single-threaded native function callable from MiraScript.
#[derive(Clone)]
pub struct MiraNativeFn {
    callback: Rc<NativeCallback>,
    name: Cow<'static, str>,
}

fn wrap_callback<V, E, F>(
    callback: F,
) -> impl Fn(&mut Runtime, &[MiraValue]) -> Result<MiraManageable> + 'static
where
    V: Into<MiraManageable>,
    E: Into<anyhow::Error>,
    F: Fn(&mut Runtime, &[MiraValue]) -> std::result::Result<V, E> + 'static,
{
    move |runtime, args| match callback(runtime, args) {
        Ok(value) => Ok(value.into()),
        Err(error) => Err(Box::<crate::MiraError>::from(error.into())),
    }
}

impl MiraNativeFn {
    /// Create a named native callback.
    pub fn new<V, E, F>(name: impl Into<Cow<'static, str>>, callback: F) -> Self
    where
        V: Into<MiraManageable>,
        E: Into<anyhow::Error>,
        F: Fn(&mut Runtime, &[MiraValue]) -> std::result::Result<V, E> + 'static,
    {
        Self {
            callback: Rc::new(wrap_callback(callback)),
            name: name.into(),
        }
    }

    /// Create an anonymous native callback.
    pub fn anonymous<V, E, F>(callback: F) -> Self
    where
        V: Into<MiraManageable>,
        E: Into<anyhow::Error>,
        F: Fn(&mut Runtime, &[MiraValue]) -> std::result::Result<V, E> + 'static,
    {
        Self {
            callback: Rc::new(wrap_callback(callback)),
            name: Cow::Borrowed(ANONYMOUS_FN_NAME),
        }
    }

    /// Create an infallible anonymous callback.
    pub fn ok<V, F>(callback: F) -> Self
    where
        V: Into<MiraManageable>,
        F: Fn(&mut Runtime, &[MiraValue]) -> V + 'static,
    {
        Self {
            callback: Rc::new(move |runtime, args| Ok(callback(runtime, args).into())),
            name: Cow::Borrowed(ANONYMOUS_FN_NAME),
        }
    }

    /// Create a callback that always returns a host error.
    pub fn err<E, F>(callback: F) -> Self
    where
        E: Into<anyhow::Error>,
        F: Fn(&mut Runtime, &[MiraValue]) -> E + 'static,
    {
        Self {
            callback: Rc::new(move |runtime, args| {
                Err(Box::<crate::MiraError>::from(
                    callback(runtime, args).into(),
                ))
            }),
            name: Cow::Borrowed(ANONYMOUS_FN_NAME),
        }
    }

    /// Create an internal callback already using the VM result type.
    pub fn builtin(
        name: impl Into<Cow<'static, str>>,
        callback: impl Fn(&mut Runtime, &[MiraValue]) -> Result<MiraManageable> + 'static,
    ) -> Self {
        Self {
            callback: Rc::new(callback),
            name: name.into(),
        }
    }

    /// Replace the diagnostic name.
    pub fn with_name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = name.into();
        self
    }
}

impl MiraFunction for MiraNativeFn {
    fn call(&self, runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraManageable> {
        (self.callback)(runtime, args)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl<V, E, F> From<F> for MiraNativeFn
where
    V: Into<MiraManageable>,
    E: Into<anyhow::Error>,
    F: Fn(&mut Runtime, &[MiraValue]) -> std::result::Result<V, E> + 'static,
{
    fn from(callback: F) -> Self {
        Self::anonymous(callback)
    }
}

impl From<MiraNativeFn> for MiraManageable {
    fn from(value: MiraNativeFn) -> Self {
        Self::from_function(value)
    }
}

impl fmt::Debug for MiraNativeFn {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("MiraNativeFn")
            .field(&self.name())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Runtime;

    #[test]
    fn test_ok_fn() {
        let mut runtime = Runtime::new();
        runtime.insert_fn(
            "add",
            MiraNativeFn::ok(|_, args| {
                let sum: f64 = args.iter().map(|arg| arg.as_number().unwrap_or(0.0)).sum();
                sum
            }),
        );

        assert_eq!(
            runtime.eval_unchecked("add(1, 2, 3)").as_number_unchecked(),
            6.0
        );
        assert_eq!(
            runtime.eval_unchecked("add(10, 20)").as_number_unchecked(),
            30.0
        );
        assert_eq!(runtime.eval_unchecked("add()").as_number_unchecked(), 0.0);
    }

    #[test]
    fn test_err_fn() {
        let mut runtime = Runtime::new();
        runtime.insert_fn(
            "fail",
            MiraNativeFn::err(|_, _| anyhow::anyhow!("This function always fails")),
        );

        assert!(matches!(
            runtime.eval("fail()").unwrap_err().as_ref(),
            crate::MiraError::External(e) if e.to_string() == "This function always fails"
        ));

        assert_eq!(
            format!(
                "{:?}",
                runtime
                    .get_function(unsafe {
                        runtime
                            .get_global("fail")
                            .unwrap()
                            .as_function()
                            .unwrap()
                            .upcast::<MiraNativeFn>()
                    })
                    .unwrap()
            ),
            "MiraNativeFn(\"fail\")"
        );
    }
}
