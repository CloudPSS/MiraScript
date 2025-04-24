use winnow::ModalResult;

use crate::error::{ErrorCode, SourceError, SourceRange};
use crate::lexer;
use crate::parser;

type CompileResult<'a> = Result<parser::Script<'a>, Vec<SourceError>>;

fn compile<'a>(
    input: &'a str,
    lexer: impl FnOnce(&mut lexer::Input<'a>) -> ModalResult<Vec<lexer::Token<'a>>>,
) -> CompileResult<'a> {
    let mut errors: Vec<_> = vec![];

    // Lexing
    let mut input = lexer::to_input(input);
    let Ok(tokens) = lexer(&mut input) else {
        errors.push(SourceError::new(
            SourceRange {
                start: 0,
                end: input.len(),
            },
            ErrorCode::LexerError,
        ));
        return Err(errors);
    };
    // Try to recover from lexing errors
    let recovered_tokens: Vec<_> = tokens
        .into_iter()
        .filter_map(|t| match t.kind {
            lexer::TokenKind::Unknown {
                recovered: Some(token),
                ..
            } => Some(lexer::Token {
                kind: *token,
                range: t.range,
                leading_trivia: t.leading_trivia,
                trailing_trivia: t.trailing_trivia,
            }),
            lexer::TokenKind::Unknown { .. } => None,
            _ => Some(t),
        })
        .collect();

    // Parsing
    let mut input = parser::to_input(&recovered_tokens);
    let Ok(script) = parser::parse(&mut input) else {
        errors.push(SourceError::new(
            SourceRange {
                start: 0,
                end: input.len(),
            },
            ErrorCode::ParserError,
        ));
        return Err(errors);
    };
    // Try to recover from parsing errors
    // let recovered_script = script
    //     .into_iter()
    //     .filter_map(|s| match s {
    //         parser::Statement::Unknown { tokens, errors } => {
    //             errors.extend(errors);
    //             Some(parser::Statement::Unknown { tokens, errors })
    //         }
    //         _ => Some(s),
    //     })
    //     .collect();
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(script)
}

#[allow(dead_code)]
pub fn compile_script(input: &str) -> CompileResult<'_> {
    compile(input, lexer::lex)
}

#[allow(dead_code)]
pub fn compile_template(input: &str) -> CompileResult<'_> {
    compile(input, lexer::lex_string)
}
