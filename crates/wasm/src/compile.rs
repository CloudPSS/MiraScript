use std::{mem::take, pin::Pin};

use mira_core::{Compiler, Config, Script, SourceDiagnostic, lexer::Token};
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

fn compile_impl(input: &str, config: &Config) -> CompileResult {
    let result = Compiler::compile(input, config);
    CompileResult {
        diagnostics: result.1,
        chunk: result.0,
    }
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

#[wasm_bindgen]
pub struct WasmCompiler {
    config: Config,
    input: Pin<String>,
    has_parse_error: bool,
    diagnostics: Vec<SourceDiagnostic>,
    tokens: Pin<Box<[Token<'static>]>>,
    script: Option<Script<'static>>,
}

#[wasm_bindgen]
impl WasmCompiler {
    #[wasm_bindgen(constructor)]
    pub fn new(input: String, config: Config) -> Self {
        Self {
            config,
            input: Pin::new(input),
            diagnostics: Vec::new(),
            has_parse_error: false,
            tokens: Box::pin([]),
            script: None,
        }
    }

    #[wasm_bindgen]
    pub fn parse(&mut self) -> bool {
        let mut compiler = Compiler::new(&self.config);
        let input: &'static str = unsafe {
            let ptr = self.input.as_ptr();
            let len = self.input.len();
            str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len))
        };
        if let Some(tokens) = compiler.lex(input) {
            self.tokens = tokens.into();
            let tokens: &'static [Token<'static>] =
                unsafe { std::slice::from_raw_parts(self.tokens.as_ptr(), self.tokens.len()) };
            if let Some(script) = compiler.parse(tokens) {
                self.script = Some(script);
                self.diagnostics = compiler.diagnostics_collector;
                return true;
            }
        }
        self.diagnostics = compiler.diagnostics_collector;
        self.has_parse_error = self.diagnostics.iter().any(|d| d.is_error());
        false
    }

    #[wasm_bindgen]
    pub fn emit(&mut self) -> Option<Vec<u8>> {
        let Some(script) = &self.script else {
            return None;
        };
        let mut compiler = Compiler::new(&self.config);
        let result = compiler.emit(script);
        self.diagnostics.append(&mut compiler.diagnostics_collector);
        result
    }

    #[wasm_bindgen]
    pub fn format(&self) -> Option<String> {
        if self.has_parse_error {
            return None;
        }
        let Some(script) = &self.script else {
            return None;
        };
        Some(mira_core::format(script, &Default::default()))
    }
}
