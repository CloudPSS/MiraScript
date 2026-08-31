use std::mem::take;

use mirascript_core::{
    CompileConfig, encode_diagnostics,
    formatter::{self, FormatEdit as CoreFormatEdit},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct FormatOptions {
    tab_size: usize,
    insert_spaces: bool,
    print_width: usize,
}

#[wasm_bindgen]
impl FormatOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let options = formatter::FormatOptions::default();
        Self {
            tab_size: options.tab_size,
            insert_spaces: options.use_spaces,
            print_width: options.line_width,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn tab_size(&self) -> usize {
        self.tab_size
    }

    #[wasm_bindgen(setter)]
    pub fn set_tab_size(&mut self, value: usize) {
        self.tab_size = value;
    }

    #[wasm_bindgen(getter)]
    pub fn insert_spaces(&self) -> bool {
        self.insert_spaces
    }

    #[wasm_bindgen(setter)]
    pub fn set_insert_spaces(&mut self, value: bool) {
        self.insert_spaces = value;
    }

    #[wasm_bindgen(getter)]
    pub fn print_width(&self) -> usize {
        self.print_width
    }

    #[wasm_bindgen(setter)]
    pub fn set_print_width(&mut self, value: usize) {
        self.print_width = value;
    }
}

impl From<&FormatOptions> for formatter::FormatOptions {
    fn from(value: &FormatOptions) -> Self {
        Self {
            tab_size: value.tab_size,
            use_spaces: value.insert_spaces,
            line_width: value.print_width,
        }
    }
}

#[wasm_bindgen]
pub struct FormatResult {
    diagnostics: Vec<u32>,
    ranges: Vec<u32>,
    texts: Vec<String>,
}

#[wasm_bindgen]
impl FormatResult {
    pub fn diagnostics(&mut self) -> Vec<u32> {
        take(&mut self.diagnostics)
    }

    pub fn ranges(&mut self) -> Vec<u32> {
        take(&mut self.ranges)
    }

    pub fn edit_count(&self) -> usize {
        self.texts.len()
    }

    pub fn edit_text(&self, index: usize) -> Option<String> {
        self.texts.get(index).cloned()
    }
}

fn utf16_to_utf8(source: &str, offset: usize) -> Option<usize> {
    let mut utf16 = 0;
    for (utf8, character) in source.char_indices() {
        if utf16 == offset {
            return Some(utf8);
        }
        utf16 += character.len_utf16();
        if utf16 > offset {
            return None;
        }
    }
    (utf16 == offset).then_some(source.len())
}

fn utf8_to_utf16(source: &str, offset: usize) -> usize {
    source[..offset].encode_utf16().count()
}

fn result(
    source: &str,
    config: &CompileConfig,
    diagnostics: Vec<mirascript_core::SourceDiagnostic>,
    edits: Vec<CoreFormatEdit>,
) -> FormatResult {
    let mut ranges = Vec::with_capacity(edits.len() * 2);
    let mut texts = Vec::with_capacity(edits.len());
    for edit in edits {
        ranges.push(utf8_to_utf16(source, edit.range.start) as u32);
        ranges.push(utf8_to_utf16(source, edit.range.end) as u32);
        texts.push(edit.text);
    }
    FormatResult {
        diagnostics: encode_diagnostics(source, &diagnostics, config),
        ranges,
        texts,
    }
}

#[wasm_bindgen]
pub fn format_sync(
    source: &str,
    config: &CompileConfig,
    options: &FormatOptions,
    ranges: &[u32],
) -> Result<FormatResult, JsError> {
    let options = options.into();
    if ranges.is_empty() {
        let outcome = formatter::format_document(source, config, &options)
            .map_err(|error| JsError::new(&error.to_string()))?;
        let edits = outcome
            .value
            .filter(|text| text != source)
            .map(|text| CoreFormatEdit {
                range: 0..source.len(),
                text,
            })
            .into_iter()
            .collect();
        return Ok(result(source, config, outcome.diagnostics, edits));
    }
    if !ranges.len().is_multiple_of(2) {
        return Err(JsError::new("format ranges must contain start/end pairs"));
    }
    let ranges = ranges
        .chunks_exact(2)
        .map(|range| {
            let start = utf16_to_utf8(source, range[0] as usize)
                .ok_or_else(|| JsError::new("invalid UTF-16 format range start"))?;
            let end = utf16_to_utf8(source, range[1] as usize)
                .ok_or_else(|| JsError::new("invalid UTF-16 format range end"))?;
            Ok(start..end)
        })
        .collect::<Result<Vec<_>, JsError>>()?;
    let outcome = formatter::format_ranges(source, config, &ranges, &options)
        .map_err(|error| JsError::new(&error.to_string()))?;
    Ok(result(
        source,
        config,
        outcome.diagnostics,
        outcome.value.unwrap_or_default(),
    ))
}
