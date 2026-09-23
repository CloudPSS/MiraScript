mod complex;
mod matrix;

pub(super) use complex::COMPLEX;
pub(super) use matrix::MATRIX;

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
