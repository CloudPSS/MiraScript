use mira_core::{Config, SourceDiagnostic};
use wasm_bindgen::prelude::*;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileFlag {
    UseUtf16,
    UseUtf32,
    HideDiagnosticError,
    HideDiagnosticWarning,
    HideDiagnosticInfo,
    HideDiagnosticHint,
    HideDiagnosticReference,
    HideDiagnosticOther,

    TrackReferences,
    Trivia,

    /// This flag is used to indicate that the maximum number of compile flags has been reached.
    #[allow(clippy::upper_case_acronyms)]
    MAX,
}

const COMPILE_FLAGS_LEN: usize = (CompileFlag::MAX as usize).div_ceil(8);
type CompileFlagsData = [u8; COMPILE_FLAGS_LEN];
struct CompileFlags<'a>(&'a CompileFlagsData);

impl<'a> CompileFlags<'a> {
    pub fn new(flags: &'a CompileFlagsData) -> Self {
        CompileFlags(flags)
    }

    pub fn get(&self, flag: CompileFlag) -> bool {
        (self.0[flag as usize / 8] & (1 << (flag as usize % 8))) != 0
    }
}

fn compile(
    script: &[u8],
    flags: &[u8],
    compile: impl FnOnce(&str, &Config) -> (Option<Box<[u8]>>, Vec<SourceDiagnostic>),
) -> CompileResult {
    let flags = CompileFlags::new(flags.try_into().expect("Invalid flags length"));
    let script = unsafe { std::str::from_utf8_unchecked(script) };

    let (chunk, diagnostics) = compile(
        script,
        &Config {
            #[cfg(feature = "track_references")]
            track_references: flags.get(CompileFlag::TrackReferences),
            #[cfg(feature = "trivia")]
            trivia: flags.get(CompileFlag::Trivia),
        },
    );
    // offsets of line starts
    let lines = script
        .lines()
        .map(|line| line.as_ptr() as usize - script.as_ptr() as usize)
        .collect::<Vec<_>>();
    let utf16 = flags.get(CompileFlag::UseUtf16);
    let utf32 = flags.get(CompileFlag::UseUtf32);
    let pos_to_line_col = |pos: usize| {
        if pos == 0 {
            return (1, 1);
        }
        let line = lines
            .iter()
            .position(|&line| line > pos)
            .unwrap_or(lines.len())
            - 1;
        let line_start = lines[line];
        let str = &script[line_start..pos];
        let col = if utf32 {
            str.chars().count()
        } else if utf16 {
            str.encode_utf16().count()
        } else {
            str.len()
        };
        (line + 1, col + 1)
    };
    let diagnostics = diagnostics
        .into_iter()
        .filter(|s| {
            if flags.get(CompileFlag::HideDiagnosticError) && s.error.is_error() {
                return false;
            }
            if flags.get(CompileFlag::HideDiagnosticWarning) && s.error.is_warning() {
                return false;
            }
            if flags.get(CompileFlag::HideDiagnosticInfo) && s.error.is_info() {
                return false;
            }
            if flags.get(CompileFlag::HideDiagnosticHint) && s.error.is_hint() {
                return false;
            }
            if flags.get(CompileFlag::HideDiagnosticReference) && s.error.is_reference() {
                return false;
            }
            if flags.get(CompileFlag::HideDiagnosticOther) && s.error.is_other() {
                return false;
            }
            true
        })
        .flat_map(|s| {
            #[cfg(debug_assertions)]
            {
                if s.range.start > s.range.end || s.range.end > script.len() {
                    panic!("Invalid diagnostic range {s}");
                }
            }
            let start = pos_to_line_col(s.range.start);
            let end = pos_to_line_col(s.range.end);
            [start.0, start.1, end.0, end.1, s.error.code() as usize]
        })
        .collect();

    CompileResult { chunk, diagnostics }
}

#[wasm_bindgen]
pub fn compile_script(script: &[u8], flags: &[u8]) -> CompileResult {
    compile(script, flags, mira_core::compile_script)
}

#[wasm_bindgen]
pub fn compile_template(script: &[u8], flags: &[u8]) -> CompileResult {
    compile(script, flags, mira_core::compile_template)
}
