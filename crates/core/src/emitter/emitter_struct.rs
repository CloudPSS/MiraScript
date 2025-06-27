use crate::{
    diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange},
    emitter::emitter_closure::Closures,
    parser::AstWalker,
};

use super::{chunk::Chunk, emitter_scope::Scopes};

pub(super) struct Emitter<'s> {
    pub chunk: Chunk<'s>,
    pub closures: Closures,
    pub scopes: Scopes<'s>,
    pub diagnostics: &'s mut Vec<SourceDiagnostic>,
}

impl<'s> Emitter<'s> {
    pub fn new(diagnostics_collector: &'s mut Vec<SourceDiagnostic>) -> Self {
        Self {
            chunk: Chunk::new(),
            closures: Closures::new(),
            scopes: Scopes::new(),
            diagnostics: diagnostics_collector,
        }
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
