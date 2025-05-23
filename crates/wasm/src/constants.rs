use strum::{VariantArray, VariantNames};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get_error_message(code: u16) -> Option<String> {
    let code: mira_core::error::ErrorCode = code.try_into().ok()?;
    code.message().to_string().into()
}

#[wasm_bindgen]
pub fn keywords() -> Vec<String> {
    mira_core::lexer::Keyword::VARIANTS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[wasm_bindgen]
pub fn control_keywords() -> Vec<String> {
    use mira_core::lexer::Keyword::*;
    [
        If, Else, Match, Case, For, While, Loop, Break, Continue, Return,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

#[wasm_bindgen]
pub fn numeric_keywords() -> Vec<String> {
    use mira_core::lexer::Keyword::*;
    [Nan, Inf].iter().map(|s| s.to_string()).collect()
}

#[wasm_bindgen]
pub fn constant_keywords() -> Vec<String> {
    use mira_core::lexer::Keyword::*;
    [True, False, Nil].iter().map(|s| s.to_string()).collect()
}

#[wasm_bindgen]
pub fn opcodes() -> Vec<String> {
    use mira_core::emitter::OpCode;
    OpCode::VARIANTS
        .iter()
        .flat_map(|s| {
            let code: u8 = (*s).into();
            [code.to_string(), s.to_string()]
        })
        .collect()
}
