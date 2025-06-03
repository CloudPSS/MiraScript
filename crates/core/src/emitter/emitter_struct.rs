use crate::diagnostic::SourceDiagnostic;

use super::{chunk::Chunk, closure::Closure, emitter_scope::Scopes};

pub(super) struct Emitter<'s> {
    pub chunk: Chunk<'s>,
    pub closures: Vec<Closure>,
    pub scopes: Scopes<'s>,
    pub diagnostics: Vec<SourceDiagnostic>,
}

impl Emitter<'_> {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            closures: vec![],
            scopes: Scopes::new(),
            diagnostics: vec![],
        }
    }
}

impl Default for Emitter<'_> {
    fn default() -> Self {
        Self::new()
    }
}
