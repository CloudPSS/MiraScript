mod utils;
use strum::VariantNames;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn keywords() -> Vec<String> {
    mira_core::lexer::Keyword::VARIANTS
        .iter()
        .map(|s| s.to_string())
        .collect()
}
