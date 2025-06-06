use crate::{
    diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange},
    parser::AstWalker,
};

use super::{chunk::Chunk, closure::Closure, emitter_scope::Scopes};

pub(super) struct Emitter<'s> {
    pub source: &'s str,
    pub chunk: Chunk<'s>,
    pub closures: Vec<Closure>,
    pub scopes: Scopes<'s>,
    pub diagnostics: Vec<SourceDiagnostic>,
}

impl<'s> Emitter<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            source,
            chunk: Chunk::new(),
            closures: vec![],
            scopes: Scopes::new(),
            diagnostics: vec![],
        }
    }

    pub fn unimplemented(
        &mut self,
        start: &(impl AstWalker<'s> + std::fmt::Debug),
        end: &(impl AstWalker<'s> + std::fmt::Debug),
    ) {
        let start_range = start.range().start;
        let end_range = end.range().end;
        if start_range == usize::MAX || end_range == usize::MIN {
            // No token in range
            return;
        }
        self.diagnostics.push(SourceDiagnostic::new(
            SourceRange {
                start: start_range,
                end: end_range,
            },
            DiagnosticCode::Unimplemented,
        ));
    }

    pub fn unreachable(
        &mut self,
        start: &(impl AstWalker<'s> + std::fmt::Debug),
        end: &(impl AstWalker<'s> + std::fmt::Debug),
        file: &str,
        line: u32,
    ) {
        let mut start_range = start.range().start;
        let mut end_range = end.range().end;
        if cfg!(debug_assertions) {
            panic!(
                "Unreachable {:?}({}) {:?}({}) at {}:{}",
                start, start_range, end, end_range, file, line
            );
        }
        if start_range == usize::MAX || end_range == usize::MIN {
            // No token in range
            start_range = 0;
            end_range = 0;
        }
        self.diagnostics.push(SourceDiagnostic::new(
            SourceRange {
                start: start_range,
                end: end_range,
            },
            DiagnosticCode::EmitterError,
        ));
    }
}
