use std::{
    num::NonZeroU64,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::bytecode::Program;

static NEXT_SCRIPT_ID: AtomicU64 = AtomicU64::new(1);

/// Stable identity shared by clones of one compiled script.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptId(NonZeroU64);

impl ScriptId {
    fn new() -> Self {
        let id = NEXT_SCRIPT_ID.fetch_add(1, Ordering::Relaxed);
        Self(NonZeroU64::new(id).expect("MiraScript identifier space exhausted"))
    }
}

/// A validated, reusable MiraScript program.
#[derive(Clone, Debug)]
pub struct MiraScript {
    id: ScriptId,
    program: Rc<Program>,
}

impl MiraScript {
    /// Return the stable identity shared by clones of this script.
    pub fn id(&self) -> ScriptId {
        self.id
    }

    pub(crate) fn program_ref(&self) -> &Program {
        &self.program
    }

    pub(crate) fn program(&self) -> Rc<Program> {
        Rc::clone(&self.program)
    }

    pub(crate) fn new(program: Program) -> Self {
        Self {
            id: ScriptId::new(),
            program: Rc::new(program),
        }
    }
}
