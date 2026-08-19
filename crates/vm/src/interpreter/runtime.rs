use std::{num::NonZeroU64, sync::atomic::AtomicU64, time::Instant};

use crate::value::MiraArena;

use super::{CallStack, FrameArena, GlobalSlots, MiraContext, Program, RunOptions};

static NEXT_EXECUTION_ID: AtomicU64 = AtomicU64::new(1);

fn next_execution_id() -> ExecutionId {
    let id = NEXT_EXECUTION_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    ExecutionId(NonZeroU64::new(id).unwrap_or_else(|| {
        panic!("Execution ID overflowed. This should never happen in practice.")
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ExecutionId(NonZeroU64);

pub struct Runtime<'a> {
    pub(crate) execution: ExecutionId,
    pub(crate) program: &'a Program,
    pub(crate) context: &'a MiraContext,
    pub(crate) options: &'a RunOptions,
    pub(super) globals: GlobalSlots<'a>,
    pub(crate) started: Instant,
    pub(crate) checkpoint_remaining: u32,
    pub(crate) call_depth: u32,
    pub(super) frames: FrameArena,
    pub(super) call_stack: CallStack,
    pub(crate) arena: MiraArena,
}

impl<'a> Runtime<'a> {
    pub(crate) fn new(
        program: &'a Program,
        context: &'a MiraContext,
        options: &'a RunOptions,
    ) -> Self {
        let execution = next_execution_id();
        Self {
            execution,
            program,
            context,
            options,
            globals: GlobalSlots::new(&program.global_names, context),
            started: Instant::now(),
            checkpoint_remaining: options.checkpoint_interval.max(1),
            call_depth: 0,
            frames: FrameArena::new(program.root.register_count),
            call_stack: CallStack::new(),
            arena: MiraArena::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<'a> Runtime<'a> {
        /// Create a new [`Runtime`] with uninitialized fields for testing purposes.
        ///
        /// # Safety
        /// The returned [`Runtime`] is uninitialized and must not be used for any operations.
        pub(crate) unsafe fn unused() -> Runtime<'static> {
            *unsafe { Box::new_zeroed().assume_init() }
        }
    }
}
