mod constants;
mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get_error_message(code: u16) -> Option<String> {
    let code: mira_core::error::ErrorCode = code.try_into().ok()?;
    code.message().to_string().into()
}

#[wasm_bindgen]
pub fn compile_script(script: &str) -> Vec<usize> {
    use mira_core::compile::compile_script;

    let (_, errors) = compile_script(script);
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
    errors
        .into_iter()
        .flat_map(|s| {
            let start = pos_to_line_col(s.range.start);
            let end = pos_to_line_col(s.range.end);
            [start.0, start.1, end.0, end.1, s.error.code() as usize]
        })
        .collect()
}
