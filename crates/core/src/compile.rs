use winnow::stream::{Location, Stream};

use crate::config::{Config, DiagnosticPositionEncoding, InputMode, set_config};
use crate::diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange};
use crate::emitter::emit;
use crate::lexer::{self, Token};
use crate::parser::{self, AstWalker};

pub type SerializedDiagnostics = Vec<u32>;
pub type CompileResult = (Option<Vec<u8>>, SerializedDiagnostics);

fn encode_diagnostic(
    script: &str,
    diagnostics: Vec<SourceDiagnostic>,
    config: &Config,
) -> SerializedDiagnostics {
    let filtered = diagnostics.into_iter().filter(|s| {
        debug_assert!(s.range.start <= s.range.end, "Invalid diagnostic range {s}");
        debug_assert!(s.range.end <= script.len(), "Invalid diagnostic range {s}");
        if config.diagnostic_error && s.error.is_error() {
            return true;
        }
        if config.diagnostic_warning && s.error.is_warning() {
            return true;
        }
        if config.diagnostic_info && s.error.is_info() {
            return true;
        }
        if config.diagnostic_hint && s.error.is_hint() {
            return true;
        }
        if config.diagnostic_reference && s.error.is_reference() {
            return true;
        }
        if config.diagnostic_other && s.error.is_other() {
            return true;
        }
        false
    });
    if config.diagnostic_position_encoding == DiagnosticPositionEncoding::None {
        filtered
            .flat_map(|s| {
                [
                    s.range.start.try_into().unwrap(),
                    s.range.end.try_into().unwrap(),
                    s.error.code() as u32,
                ]
            })
            .collect()
    } else {
        let mut pos_to_line_col = {
            // offsets of line starts
            let mut lines = Vec::new();
            let utf16 = config.diagnostic_position_encoding == DiagnosticPositionEncoding::Utf16;
            let utf32 = config.diagnostic_position_encoding == DiagnosticPositionEncoding::Utf32;
            move |pos: usize| {
                if pos == 0 {
                    return (1u32, 1u32);
                }
                if lines.is_empty() {
                    lines.extend(
                        script
                            .lines()
                            .map(|line| line.as_ptr() as usize - script.as_ptr() as usize),
                    );
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
                (
                    (line + 1).try_into().unwrap(),
                    (col + 1).try_into().unwrap(),
                )
            }
        };
        filtered
            .flat_map(|s| {
                let start = pos_to_line_col(s.range.start);
                let end = pos_to_line_col(s.range.end);
                [start.0, start.1, end.0, end.1, s.error.code() as u32]
            })
            .collect()
    }
}

fn recover_token<'s>(
    mut t: Token<'s>,
    diagnostics_collector: &mut Vec<SourceDiagnostic>,
) -> Option<Token<'s>> {
    match t.kind {
        lexer::TokenKind::Unknown {
            recovered: Some(token),
            errors,
        } => {
            diagnostics_collector.extend(errors);
            t.kind = *token;
            recover_token(t, diagnostics_collector)
        }
        lexer::TokenKind::Unknown { errors, .. } => {
            diagnostics_collector.extend(errors);
            None
        }
        lexer::TokenKind::InterpolatedString(ref mut v) => {
            diagnostics_collector.push(SourceDiagnostic::new(
                t.range.clone(),
                DiagnosticCode::Interpolation,
            ));
            for (_, tokens) in v.iter_mut() {
                *tokens = tokens
                    .iter_mut()
                    .filter_map(|t| recover_token(t.clone(), diagnostics_collector))
                    .collect::<Vec<_>>();
            }
            Some(t)
        }
        lexer::TokenKind::String(_) => {
            diagnostics_collector.push(SourceDiagnostic::new(
                t.range.clone(),
                DiagnosticCode::String,
            ));
            Some(t)
        }
        _ => Some(t),
    }
}

pub fn compile(input: &str, config: &Config) -> CompileResult {
    set_config(config);
    let mut diagnostics_collector: Vec<_> = vec![];
    let encode_diags = move |diagnostics_collector: Vec<SourceDiagnostic>| {
        encode_diagnostic(input, diagnostics_collector, config)
    };

    // Lexing
    let tokens = {
        let current_lexer = if config.input_mode == InputMode::Script {
            lexer::lex
        } else {
            lexer::lex_string
        };
        let mut input = lexer::to_input(input);
        let result = current_lexer(&mut input);
        if result.is_err() {
            let remaining = input.peek_finish();
            let range = if remaining.is_empty() {
                SourceRange {
                    start: 0,
                    end: input.len(),
                }
            } else {
                SourceRange {
                    start: input.current_token_start(),
                    end: input.len(),
                }
            };
            diagnostics_collector.push(SourceDiagnostic::new(range, DiagnosticCode::LexerError));
            return (None, encode_diags(diagnostics_collector));
        }
        result.unwrap()
    };
    // Try to recover from lexing errors
    let recovered_tokens: Vec<_> = tokens
        .into_iter()
        .filter_map(|t| recover_token(t, &mut diagnostics_collector))
        .collect();

    // Parsing
    let mut script = {
        let mut tokens = parser::to_input(&recovered_tokens);
        let result = parser::parse(&mut tokens);
        if result.is_err() {
            let remaining = tokens.peek_finish();
            let range = if remaining.is_empty() {
                SourceRange {
                    start: 0,
                    end: input.len(),
                }
            } else {
                SourceRange {
                    start: remaining.first().unwrap().range.start,
                    end: remaining.last().unwrap().range.end,
                }
            };
            diagnostics_collector.push(SourceDiagnostic::new(range, DiagnosticCode::ParserError));
            return (None, encode_diags(diagnostics_collector));
        }
        result.unwrap()
    };

    // collect and recover from parsing errors
    script.collect_diagnostics(&mut diagnostics_collector);

    // Emitting
    let bytecode = emit(input, &script, &mut diagnostics_collector);

    let has_error = diagnostics_collector.iter().any(|d| d.error.is_error());
    let diagnostics = encode_diags(diagnostics_collector);

    if has_error {
        return (None, diagnostics);
    }

    (Some(bytecode), diagnostics)
}
