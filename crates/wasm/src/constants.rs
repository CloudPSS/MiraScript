use mira_core::diagnostic::DiagnosticCode;
use mira_core::lexer::Keyword;
use strum::VariantArray;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get_diagnostic_message(code: DiagnosticCode) -> Option<String> {
    code.message().to_string().into()
}

#[wasm_bindgen]
pub fn keywords() -> Vec<String> {
    Keyword::VARIANTS.iter().map(|s| s.to_string()).collect()
}

#[wasm_bindgen]
pub fn control_keywords() -> Vec<String> {
    Keyword::VARIANTS
        .iter()
        .filter_map(|s| {
            if s.is_control() {
                Some(s.to_string())
            } else {
                None
            }
        })
        .collect()
}

#[wasm_bindgen]
pub fn numeric_keywords() -> Vec<String> {
    Keyword::VARIANTS
        .iter()
        .filter_map(|s| {
            if s.is_numeric() {
                Some(s.to_string())
            } else {
                None
            }
        })
        .collect()
}

#[wasm_bindgen]
pub fn constant_keywords() -> Vec<String> {
    Keyword::VARIANTS
        .iter()
        .filter_map(|s| {
            if s.is_constant() {
                Some(s.to_string())
            } else {
                None
            }
        })
        .collect()
}

#[wasm_bindgen]
pub fn reserved_keywords() -> Vec<String> {
    Keyword::VARIANTS
        .iter()
        .filter_map(|s| {
            if s.is_reserved() {
                Some(s.to_string())
            } else {
                None
            }
        })
        .collect()
}
