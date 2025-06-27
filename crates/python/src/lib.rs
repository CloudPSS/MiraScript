use mira_core::{
    config::{Config as ConfigData, InputMode},
    Compiler, DiagnosticCode,
};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[pyclass]
#[derive(Debug, Default)]
struct Config {
    pub data: ConfigData,
}

#[pymethods]
impl Config {
    #[new]
    #[pyo3(signature = (**data))]
    fn new(data: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let Some(data) = data else {
            return Ok(Config::default());
        };

        let mut config = ConfigData::new();
        if let Some(input_mode) = data.get_item("input_mode")? {
            config.input_mode = input_mode
                .extract::<String>()
                .ok()
                .and_then(|s| match s.as_str() {
                    "script" => Some(InputMode::Script),
                    "template" => Some(InputMode::Template),
                    _ => None,
                })
                .ok_or_else(|| {
                    PyValueError::new_err("Invalid input_mode, expected 'script' or 'template'")
                })?;
        }
        Ok(Config { data: config })
    }
}

#[pyfunction]
fn compile(script: String, config: PyRef<'_, Config>) -> PyResult<(Option<Vec<u8>>, Vec<u32>)> {
    let config: &ConfigData = &config.data;
    let (chunk, diagnostics) = Compiler::compile(&script, config);
    Ok((chunk, diagnostics))
}

#[pyfunction]
fn get_diagnostic_message(code: u16) -> PyResult<(&'static str, &'static str, &'static str)> {
    match DiagnosticCode::try_from(code) {
        Ok(error) => Ok((error.level_str(), error.into(), error.message())),
        Err(_) => Err(PyValueError::new_err(format!(
            "No diagnostic message found for code {code}"
        ))),
    }
}

#[pymodule]
fn mirascript(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_class::<Config>()?;
    m.add_function(wrap_pyfunction!(get_diagnostic_message, m)?)?;
    Ok(())
}
