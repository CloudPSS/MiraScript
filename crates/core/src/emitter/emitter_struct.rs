use crate::{
    diagnostic::{DiagnosticCode, SourceDiagnostic, SourceRange},
    emitter::emitter_closure::Closures,
    parser::AstWalker,
};

use super::{chunk::Chunk, emitter_scope::Scopes};

pub(super) struct Emitter<'s> {
    pub source: &'s str,
    pub chunk: Chunk<'s>,
    pub closures: Closures,
    pub scopes: Scopes<'s>,
    pub diagnostics: Vec<SourceDiagnostic>,
}

impl<'s> Emitter<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            source,
            chunk: Chunk::new(),
            closures: Closures::new(),
            scopes: Scopes::new(),
            diagnostics: vec![],
        }
    }

    pub fn unimplemented(
        &mut self,
        start: &(impl AstWalker<'s> + std::fmt::Debug),
        end: &(impl AstWalker<'s> + std::fmt::Debug),
    ) {
        let start_range = start.range();
        let end_range = end.range();
        let start;
        let end;
        if start_range.start != usize::MAX {
            start = start_range.start;
        } else if end_range.start != usize::MAX {
            start = end_range.start;
        } else {
            return;
        }
        if end_range.end != usize::MIN {
            end = end_range.end;
        } else if start_range.end != usize::MIN {
            end = start_range.end;
        } else {
            return;
        }
        self.diagnostics.push(SourceDiagnostic::new(
            start..end,
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
