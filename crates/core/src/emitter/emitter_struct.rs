use crate::error::SourceError;

use super::{chunk::Chunk, closure::Closure, emitter_scope::Scopes, scope::Scope};

pub(super) struct Emitter<'s> {
    pub chunk: Chunk<'s>,
    pub closures: Vec<Closure>,
    pub scopes: Scopes<'s>,
    pub errors: Vec<SourceError>,
}

impl<'s> Emitter<'s> {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            closures: vec![],
            scopes: Scopes::new(),
            errors: vec![],
        }
    }
}
