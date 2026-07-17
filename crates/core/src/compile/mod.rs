use winnow::stream::{Location, Stream};

use crate::parser::{self, AstWalker};
use crate::{
    Script,
    config::{Config, InputMode},
};
use crate::{diagnostic::DiagnosticsCollector, emitter};
use crate::{
    diagnostic::SerializedDiagnostics,
    lexer::{self},
};
use crate::{
    diagnostic::{DiagnosticCode, SourceRange},
    lexer::{Token, TokenArena},
};

mod recover_token;

pub type CompileResult = (Option<Vec<u8>>, SerializedDiagnostics);

pub struct Compiler<'s, 'c> {
    pub script: &'s str,
    pub config: &'c Config,
    pub diagnostics_collector: DiagnosticsCollector<'s, 'c>,
}

impl<'s, 'c: 's> Compiler<'s, 'c> {
    pub fn new(script: &'s str, config: &'c Config) -> Self {
        Self {
            script,
            config,
            diagnostics_collector: DiagnosticsCollector::new(config, script),
        }
    }

    pub fn lex(&mut self) -> Option<TokenArena<'s>> {
        let current_lexer = if self.config.input_mode == InputMode::Script {
            lexer::lex
        } else {
            lexer::lex_string
        };
        let mut input = lexer::to_input(self.script, self.config);
        let Ok(tokens) = current_lexer(&mut input) else {
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
            self.diagnostics_collector
                .push(DiagnosticCode::LexerError, range);
            return None;
        };

        // Try to recover from lexing errors
        let mut arena = TokenArena::with_capacity(tokens.len());
        for token in tokens
            .into_iter()
            .filter_map(|t| recover_token::recover_token(t, &mut self.diagnostics_collector))
        {
            arena.alloc(token);
        }
        Some(arena)
    }

    pub fn parse<'t>(&mut self, tokens: &'t TokenArena<'t>) -> Option<Script<'t>> {
        let tokens = tokens.as_slice();
        assert!(!tokens.is_empty(), "Cannot parse an empty token list");

        // Parsing
        let mut stream = parser::to_input(tokens);
        let Ok(mut script) = parser::parse(&mut stream) else {
            let remaining = stream.peek_finish();
            let range = if remaining.is_empty() {
                tokens.last().unwrap().range.clone()
            } else {
                SourceRange {
                    start: remaining.first().unwrap().range.start,
                    end: remaining.last().unwrap().range.end,
                }
            };
            self.diagnostics_collector
                .push(DiagnosticCode::ParserError, range);
            return None;
        };

        // collect and recover from parsing errors
        script.collect_diagnostics(&mut self.diagnostics_collector);
        Some(script)
    }

    pub fn emit(&mut self, script: &Script<'s>) -> Option<Vec<u8>> {
        // Emitting
        let bytecode = emitter::emit(script, &mut self.diagnostics_collector);

        let has_error = self
            .diagnostics_collector
            .iter()
            .any(|d| d.error.is_error());

        if has_error {
            return None;
        }

        Some(bytecode)
    }

    pub fn encode_diagnostics(&self) -> SerializedDiagnostics {
        self.diagnostics_collector.encode()
    }

    pub fn compile(input: &str, config: &Config) -> CompileResult {
        let mut compiler = Compiler::new(input, config);
        let Some(tokens) = compiler.lex() else {
            return (None, compiler.encode_diagnostics());
        };
        let Some(script) = compiler.parse(&tokens) else {
            return (None, compiler.encode_diagnostics());
        };
        let Some(chunk) = compiler.emit(&script) else {
            return (None, compiler.encode_diagnostics());
        };
        (Some(chunk), compiler.encode_diagnostics())
    }
}
