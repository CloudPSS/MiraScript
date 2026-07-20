use bumpalo::Bump;

/// Arena allocator for AST nodes. All `bumpalo::boxed::Box<'a, T>` allocations
/// made through this arena live as long as the arena itself.
pub struct AstArena {
    bump: Bump,
}

impl AstArena {
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    pub fn alloc<'a, T>(&'a self, value: T) -> bumpalo::boxed::Box<'a, T> {
        bumpalo::boxed::Box::new_in(value, &self.bump)
    }

    pub fn reset(&mut self) {
        self.bump.reset();
    }
}

impl Default for AstArena {
    fn default() -> Self {
        Self::new()
    }
}
