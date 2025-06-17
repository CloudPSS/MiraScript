use std::mem::take;

use mira_core::Config;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct CompileResult {
    chunk: Option<Vec<u8>>,
    diagnostics: Vec<u32>,
}

#[wasm_bindgen]
impl CompileResult {
    pub fn chunk(&mut self) -> Option<Vec<u8>> {
        take(&mut self.chunk)
    }

    pub fn diagnostics(&mut self) -> Vec<u32> {
        take(&mut self.diagnostics)
    }
}

fn compile_impl(script: &str, config: &Config) -> CompileResult {
    let (chunk, diagnostics) = mira_core::compile(script, config);
    CompileResult { chunk, diagnostics }
}

#[wasm_bindgen]
pub unsafe fn compile_buffer(script: &[u8], config: &Config) -> CompileResult {
    let script = unsafe { std::str::from_utf8_unchecked(script) };
    compile_impl(script, config)
}

#[wasm_bindgen]
pub fn compile(script: &str, config: &Config) -> CompileResult {
    compile_impl(script, config)
}
