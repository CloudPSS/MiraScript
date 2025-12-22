/// Mode for reading input.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    any(feature = "wasm", feature = "wasm-constants"),
    wasm_bindgen::prelude::wasm_bindgen
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Script,
    Template,
}

/// Encoding for counting positions in diagnostics.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    any(feature = "wasm", feature = "wasm-constants"),
    wasm_bindgen::prelude::wasm_bindgen
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticPositionEncoding {
    /// Use the default encoding (UTF-8) and
    /// 0-based indexing from the start of the file.
    /// Do not convert positions to line-column format.
    None,
    /// Convert positions to 1-based UTF-8 line-column format.
    Utf8,
    /// Convert positions to 1-based UTF-16 line-column format.
    Utf16,
    /// Convert positions to 1-based UTF-32 line-column format.
    Utf32,
}

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(default)
)]
#[cfg_attr(
    any(feature = "wasm", feature = "wasm-constants"),
    wasm_bindgen::prelude::wasm_bindgen
)]
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    #[cfg(feature = "formatter")]
    pub trivia: bool,

    pub input_mode: InputMode,

    pub diagnostic_position_encoding: DiagnosticPositionEncoding,
    pub diagnostic_error: bool,
    pub diagnostic_warning: bool,
    pub diagnostic_info: bool,
    pub diagnostic_hint: bool,
    pub diagnostic_reference: bool,
    pub diagnostic_tag: bool,
    pub diagnostic_sourcemap: bool,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Config {
    #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(constructor))]
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "formatter")]
            trivia: false,
            input_mode: InputMode::Script,
            diagnostic_position_encoding: DiagnosticPositionEncoding::Utf8,
            diagnostic_error: true,
            diagnostic_warning: true,
            diagnostic_info: true,
            diagnostic_hint: true,
            diagnostic_reference: true,
            diagnostic_tag: false,
            diagnostic_sourcemap: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "wasm")]
mod wasm;
