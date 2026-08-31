use std::pin::Pin;

use mirascript_core::{
    CompileConfig, Compiler, Script, SourceDiagnostic, Token, encode_diagnostics,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct MonacoCompiler {
    config: CompileConfig,
    input: Pin<String>,
    has_parse_error: bool,
    diagnostics: Vec<SourceDiagnostic>,
    tokens: Pin<Box<[Token<'static>]>>,
    script: Option<Script<'static>>,
}

#[wasm_bindgen]
impl MonacoCompiler {
    #[wasm_bindgen(constructor)]
    pub fn new(input: String, config: &CompileConfig) -> Self {
        Self {
            config: config.clone(),
            input: Pin::new(input),
            diagnostics: Vec::new(),
            has_parse_error: false,
            tokens: Box::pin([]),
            script: None,
        }
    }

    #[wasm_bindgen]
    pub fn parse(&mut self) -> bool {
        let input: &'static str = unsafe {
            let ptr = self.input.as_ptr();
            let len = self.input.len();
            str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len))
        };
        let config: &'static CompileConfig = unsafe { &*(&self.config as *const CompileConfig) };
        let mut compiler = Compiler::new(input, config);
        if let Some(tokens) = compiler.lex() {
            self.tokens = tokens.into();
            let tokens =
                unsafe { std::slice::from_raw_parts(self.tokens.as_ptr(), self.tokens.len()) };
            if let Some(script) = compiler.parse(tokens) {
                self.script = Some(script);
                self.diagnostics = compiler.diagnostics_collector.drain(..).collect();
                self.has_parse_error = self.diagnostics.iter().any(|d| d.is_error());
                return true;
            }
        }
        self.diagnostics = compiler.diagnostics_collector.drain(..).collect();
        self.has_parse_error = self.diagnostics.iter().any(|d| d.is_error());
        false
    }

    #[wasm_bindgen]
    pub fn emit(&mut self) -> Option<Vec<u8>> {
        let Some(script) = &self.script else {
            return None;
        };
        let mut compiler = Compiler::new(&self.input, &self.config);
        let result = compiler.emit(script);
        self.diagnostics.append(&mut compiler.diagnostics_collector);
        result
    }

    #[wasm_bindgen]
    pub fn diagnostics(&self) -> Vec<u32> {
        encode_diagnostics(&self.input, &self.diagnostics, &self.config)
    }
}
