use winnow::stream::{Location, Stream};

use crate::emitter;
use crate::lexer::{self};
use crate::parser::{self, AstWalker};
use crate::{
    Script,
    config::{Config, InputMode, set_config},
};
use crate::{
    diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange},
    lexer::Token,
};

mod encode_diagnostics;
mod recover_token;

pub type SerializedDiagnostics = Vec<u32>;
pub type CompileResult = (Option<Vec<u8>>, SerializedDiagnostics);
pub use encode_diagnostics::encode_diagnostics;

pub struct Compiler<'c> {
    pub config: &'c Config,
    pub diagnostics_collector: Vec<SourceDiagnostic>,
}

impl<'c> Compiler<'c> {
    pub fn new(config: &'c Config) -> Self {
        Self {
            config,
            diagnostics_collector: Vec::new(),
        }
    }

    pub fn lex<'s>(&mut self, input: &'s str) -> Option<Box<[Token<'s>]>> {
        let config = self.config;
        set_config(config);

        let current_lexer = if config.input_mode == InputMode::Script {
            lexer::lex
        } else {
            lexer::lex_string
        };
        let mut input = lexer::to_input(input);
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
                .push(SourceDiagnostic::new(range, DiagnosticCode::LexerError));
            return None;
        };

        // Try to recover from lexing errors
        Some(
            tokens
                .into_iter()
                .filter_map(|t| recover_token::recover_token(t, &mut self.diagnostics_collector))
                .collect(),
        )
    }

    pub fn parse<'s>(&mut self, tokens: &'s [Token<'s>]) -> Option<Script<'s>> {
        assert!(!tokens.is_empty(), "Cannot parse an empty token list");

        set_config(self.config);

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
                .push(SourceDiagnostic::new(range, DiagnosticCode::ParserError));
            return None;
        };

        // collect and recover from parsing errors
        script.collect_diagnostics(&mut self.diagnostics_collector);
        Some(script)
    }

    pub fn emit(&mut self, script: &Script<'_>) -> Option<Vec<u8>> {
        set_config(self.config);

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

    pub fn encode_diagnostics(&self, input: &str) -> SerializedDiagnostics {
        encode_diagnostics(input, &self.diagnostics_collector, self.config)
    }

    pub fn compile(input: &str, config: &Config) -> CompileResult {
        let mut compiler = Compiler::new(config);
        if let Some(tokens) = compiler.lex(input) {
            if let Some(script) = compiler.parse(&tokens) {
                if let Some(chunk) = compiler.emit(&script) {
                    return (Some(chunk), compiler.encode_diagnostics(input));
                }
            }
        }
        (None, compiler.encode_diagnostics(input))
    }
}
