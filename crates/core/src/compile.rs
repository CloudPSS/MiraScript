use std::cell::RefCell;

use winnow::ModalResult;
use winnow::stream::{Location, Stream};

use crate::diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange};
use crate::emitter::emit;
use crate::lexer::{self, Token, TokenKind};
use crate::parser::{self, AstWalker, walker_mut};

type CompileResult<'s> = (Option<Box<[u8]>>, Vec<SourceDiagnostic>);

fn recover_token<'s>(
    t: Token<'s>,
    diagnostics_collector: &mut Vec<SourceDiagnostic>,
) -> Option<Token<'s>> {
    match t.kind {
        lexer::TokenKind::Unknown {
            recovered: Some(token),
            errors,
        } => {
            diagnostics_collector.extend(errors);
            let recovered = lexer::Token {
                kind: *token,
                range: t.range,
                leading_trivia: t.leading_trivia,
                trailing_trivia: t.trailing_trivia,
            };
            recover_token(recovered, diagnostics_collector)
        }
        lexer::TokenKind::Unknown { errors, .. } => {
            diagnostics_collector.extend(errors);
            None
        }
        lexer::TokenKind::InterpolatedString(strs, interpolations) => {
            diagnostics_collector.push(SourceDiagnostic::new(
                t.range.clone(),
                DiagnosticCode::Interpolation,
            ));
            Some(Token {
                kind: TokenKind::InterpolatedString(
                    strs,
                    interpolations
                        .into_iter()
                        .map(|ts| {
                            ts.into_iter()
                                .filter_map(|t| recover_token(t, diagnostics_collector))
                                .collect()
                        })
                        .collect(),
                ),
                range: t.range,
                leading_trivia: t.leading_trivia,
                trailing_trivia: t.trailing_trivia,
            })
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
    lexer: impl FnOnce(&mut lexer::Input<'s>) -> ModalResult<Vec<lexer::Token<'s>>>,
) -> CompileResult<'s> {
    let mut diagnostics_collector: Vec<_> = vec![];

    // Lexing
    let tokens = {
        let mut input = lexer::to_input(input);
        let result = lexer(&mut input);
        if let Err(err) = result {
            eprintln!("Lexer error: {:?} {:?}", err, input);
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
        if let Err(err) = result {
            eprintln!("Parser error: {:?} {:?}", err, tokens);
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
    {
        let diagnostics_collector = RefCell::new(&mut diagnostics_collector);
        let mut w = walker_mut(
            |token| {
                let Token {
                    kind: TokenKind::Unknown { recovered, errors },
                    range,
                    leading_trivia,
                    trailing_trivia,
                } = token
                else {
                    return;
                };
                diagnostics_collector.borrow_mut().extend(errors.drain(..));
                if let Some(recovered) = std::mem::take(recovered) {
                    *token = Token {
                        kind: *recovered,
                        range: range.clone(),
                        leading_trivia: std::mem::take(leading_trivia),
                        trailing_trivia: std::mem::take(trailing_trivia),
                    };
                }
            },
            |expression| {
                if let parser::Expression::Unknown { errors, .. } = expression {
                    diagnostics_collector.borrow_mut().extend(errors.drain(..));
                }
            },
            |pattern| {
                if let parser::Pattern::Unknown { errors, .. } = pattern {
                    diagnostics_collector.borrow_mut().extend(errors.drain(..));
                }
            },
            |statement| {
                if let parser::Statement::Unknown { errors, .. } = statement {
                    diagnostics_collector.borrow_mut().extend(errors.drain(..));
                }
            },
        );
        // Try to recover from parsing errors
        script.walk_mut(&mut w);
    }

    // Emitting
    let (diagnostics, bytecode) = emit(input, &script);
    diagnostics_collector.extend(diagnostics);

    if diagnostics_collector.iter().any(|e| e.is_error()) {
        return (None, diagnostics_collector);
    }

    (Some(bytecode), diagnostics_collector)
}

#[allow(dead_code)]
pub fn compile_script(input: &str) -> CompileResult<'_> {
    compile(input, lexer::lex)
}

#[allow(dead_code)]
pub fn compile_template(input: &str) -> CompileResult<'_> {
    compile(input, lexer::lex_string)
}
