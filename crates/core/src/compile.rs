use std::cell::RefCell;

use winnow::ModalResult;
use winnow::stream::{Location, Stream};

use crate::emitter::emit;
use crate::error::{ErrorCode, SourceError, SourceRange};
use crate::lexer::{self, Token, TokenKind};
use crate::parser::{self, AstWalker, walker_mut};

type CompileResult<'s> = (Option<Box<[u8]>>, Vec<SourceError>);

fn compile<'s>(
    input: &'s str,
    lexer: impl FnOnce(&mut lexer::Input<'s>) -> ModalResult<Vec<lexer::Token<'s>>>,
) -> CompileResult<'s> {
    let mut error_collector: Vec<_> = vec![];

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
            error_collector.push(SourceError::new(range, ErrorCode::LexerError));
            return (None, error_collector);
        }
        result.unwrap()
    };
    // Try to recover from lexing errors
    let recovered_tokens: Vec<_> = tokens
        .into_iter()
        .filter_map(|t| match t.kind {
            lexer::TokenKind::Unknown {
                recovered: Some(token),
                errors,
            } => {
                error_collector.extend(errors);
                Some(lexer::Token {
                    kind: *token,
                    range: t.range,
                    leading_trivia: t.leading_trivia,
                    trailing_trivia: t.trailing_trivia,
                })
            }
            lexer::TokenKind::Unknown { errors, .. } => {
                error_collector.extend(errors);
                None
            }
            _ => Some(t),
        })
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
            error_collector.push(SourceError::new(range, ErrorCode::ParserError));
            return (None, error_collector);
        }
        result.unwrap()
    };

    // collect and recover from parsing errors
    {
        let error_collector = RefCell::new(&mut error_collector);
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
                error_collector.borrow_mut().extend(errors.drain(..));
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
                    error_collector.borrow_mut().extend(errors.drain(..));
                }
            },
            |pattern| {
                if let parser::Pattern::Unknown { errors, .. } = pattern {
                    error_collector.borrow_mut().extend(errors.drain(..));
                }
            },
            |statement| {
                if let parser::Statement::Unknown { errors, .. } = statement {
                    error_collector.borrow_mut().extend(errors.drain(..));
                }
            },
        );
        // Try to recover from parsing errors
        script.walk_mut(&mut w);
    }

    // Emitting
    let (errors, bytecode) = emit(&script);
    error_collector.extend(errors);

    if error_collector.iter().any(|e| e.is_error()) {
        return (None, error_collector);
    }

    (Some(bytecode), error_collector)
}

#[allow(dead_code)]
pub fn compile_script(input: &str) -> CompileResult<'_> {
    compile(input, lexer::lex)
}

#[allow(dead_code)]
pub fn compile_template(input: &str) -> CompileResult<'_> {
    compile(input, lexer::lex_string)
}
