use crate::mira;

#[path = "matrix/mod.rs"]
mod matrix_impl;
#[mira]
pub(super) mod matrix {
    use crate::standard_library::{callable, const_value, required};
    use crate::{MiraValue, Result, Runtime};

    use super::matrix_impl as i;

    #[mira]
    fn zeros(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::filled::<0>(runtime, args)
    }

    #[mira]
    fn ones(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::filled::<1>(runtime, args)
    }

    #[mira]
    fn identity(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::identity(runtime, args)
    }

    #[mira]
    fn diagonal(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::diagonal(runtime, args)
    }

    #[mira]
    fn size(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::size(runtime, args)
    }

    #[mira]
    fn transpose(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::transpose(runtime, args)
    }

    #[mira]
    fn invert(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::invert(runtime, args)
    }

    #[mira]
    fn add(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::numeric_entrywise(runtime, args, |a, b| a + b)
    }

    #[mira]
    fn subtract(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::numeric_entrywise(runtime, args, |a, b| a - b)
    }

    #[mira]
    fn entrywise_multiply(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::numeric_entrywise(runtime, args, |a, b| a * b)
    }

    #[mira]
    fn entrywise_divide(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::numeric_entrywise(runtime, args, |a, b| a / b)
    }

    #[mira]
    fn multiply(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        i::multiply(runtime, args)
    }

    #[mira]
    fn entrywise(runtime: &mut Runtime, args: &[MiraValue]) -> Result<MiraValue> {
        let left = required(args, 0, "a")?;
        let right = required(args, 1, "b")?;
        let function = callable(args, 2, "f")?;
        i::entrywise(runtime, *left, *right, &mut |runtime, a, b| {
            runtime.checkpoint()?;
            const_value(function.call(runtime, &[a, b])?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Runtime;

    #[test]
    fn generated_matrix_names_are_distinct_from_export_keys() {
        let mut runtime = Runtime::new();
        let module = runtime.get_global("matrix").unwrap().as_module().unwrap();
        let module = runtime.get_module_dyn(module).unwrap();
        assert_eq!(module.name(), "matrix");
        assert_eq!(module.index_of("add"), Some(7));

        let function = runtime.eval_unchecked("matrix.add");
        let function = runtime
            .get_function_dyn(function.as_function().unwrap())
            .unwrap();
        assert_eq!(function.name(), "matrix.add");
    }
}
