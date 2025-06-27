use mira_core::{
    config::{Config as ConfigData, InputMode},
    Compiler,
};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// MiraScript 编译配置
///
/// Args:
///     **input_mode (str): 输入模式，支持 'script' 和 'template'
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

/// 编译 MiraScript 代码，生成字节码
///
/// Args:
///     script (str): 要编译的 MiraScript 代码
///     config (Config): 编译配置
///
/// Returns:
///    (Optional[bytes], List[int]): 编译后的字节码和诊断信息
#[pyfunction]
fn compile(script: String, config: PyRef<'_, Config>) -> PyResult<(Option<Vec<u8>>, Vec<u32>)> {
    let config: &ConfigData = &config.data;
    let (chunk, diagnostics) = Compiler::compile(&script, config);
    Ok((chunk, diagnostics))
}

/// MiraScript Python 模块
#[pymodule]
fn mirascript(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_class::<Config>()?;
    Ok(())
}
