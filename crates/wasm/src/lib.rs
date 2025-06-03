use wasm_bindgen::prelude::*;

mod constants;
mod utils;

#[wasm_bindgen]
pub struct CompileResult {
    chunk: Option<Box<[u8]>>,
    diagnostics: Box<[usize]>,
}

#[wasm_bindgen]
impl CompileResult {
    pub fn chunk(&self) -> Option<Box<[u8]>> {
        self.chunk.clone()
    }

    pub fn diagnostics(&self) -> Box<[usize]> {
        self.diagnostics.clone()
    }
}

#[wasm_bindgen]
pub fn compile_script(script: &str) -> CompileResult {
    use mira_core::compile::compile_script;

    let (chunk, diagnostics) = compile_script(script);
    // offsets of line starts
    let lines = script
        .lines()
        .map(|line| line.as_ptr() as usize - script.as_ptr() as usize)
        .collect::<Vec<_>>();
    let pos_to_line_col = |pos: usize| {
        let line = lines
            .iter()
            .position(|&line| line > pos)
            .unwrap_or(lines.len())
            - 1;
        let str = &script[lines[line]..pos];
        let col = str.encode_utf16().count();
        (line + 1, col + 1)
    };
    let diagnostics = diagnostics
        .into_iter()
        .flat_map(|s| {
            let start = pos_to_line_col(s.range.start);
            let end = pos_to_line_col(s.range.end);
            [start.0, start.1, end.0, end.1, s.error.code() as usize]
        })
        .collect();

    CompileResult { chunk, diagnostics }
}
