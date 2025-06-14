use winnow::stream::{Location, Stream};

use crate::config::{Config, set_config};
use crate::diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange};
use crate::emitter::emit;
use crate::lexer::{self, Token};
use crate::parser::{self, AstWalker};

type CompileResult<'s> = (Option<Box<[u8]>>, Vec<SourceDiagnostic>);

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

fn compile<'s>(
    input: &'s str,
    lexer: impl FnOnce(&mut lexer::Input<'s>) -> lexer::Result<Vec<lexer::Token<'s>>>,
    config: &Config,
) -> CompileResult<'s> {
    set_config(config);
    let mut diagnostics_collector: Vec<_> = vec![];

    // Lexing
    let tokens = {
        let mut input = lexer::to_input(input);
        let result = lexer(&mut input);
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
            return (None, diagnostics_collector);
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
            return (None, diagnostics_collector);
        }
        result.unwrap()
    };

    // collect and recover from parsing errors
    script.collect_diagnostics(&mut diagnostics_collector);

    // Emitting
    let (diagnostics, bytecode) = emit(input, &script);
    diagnostics_collector.extend(diagnostics);

    if diagnostics_collector.iter().any(|e| e.is_error()) {
        return (None, diagnostics_collector);
    }

    (Some(bytecode), diagnostics_collector)
}

#[allow(dead_code)]
pub fn compile_script<'s>(input: &'s str, config: &Config) -> CompileResult<'s> {
    compile(input, lexer::lex, config)
}

#[allow(dead_code)]
pub fn compile_template<'s>(input: &'s str, config: &Config) -> CompileResult<'s> {
    compile(input, lexer::lex_string, config)
}
