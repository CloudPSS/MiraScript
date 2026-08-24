use super::{MiraAny, register::Registers};

/// Identifies a frame in the call stack.
/// Root frame is always `0`, and child frames are numbered sequentially starting from `1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct FrameId(pub(super) usize);

impl FrameId {
    pub(super) const ROOT: Self = Self(0);
}

#[derive(Debug)]
pub(super) struct Frame {
    pub(super) parent: Option<FrameId>,
    pub(super) registers: Registers,
}

impl Frame {
    pub fn new(register_count: usize, parent: Option<FrameId>) -> Self {
        Self {
            registers: Registers::new(register_count),
            parent,
        }
    }

    pub fn reset(&mut self, parent: Option<FrameId>) {
        self.registers.fill(MiraAny::uninitialized());
        self.parent = parent;
    }

    fn reset_with_count(&mut self, register_count: usize, parent: Option<FrameId>) {
        self.registers.reset(register_count);
        self.parent = parent;
    }
}

pub(super) struct FrameArena {
    root: Frame,
    children: Vec<Frame>,
    active_children: usize,
}

impl FrameArena {
    pub(super) fn new(root_register_count: usize) -> Self {
        Self {
            root: Frame::new(root_register_count, None),
            children: Vec::new(),
            active_children: 0,
        }
    }

    pub(super) fn begin_run(&mut self, root_register_count: usize) {
        self.root.reset_with_count(root_register_count, None);
        // Inactive frames retain capacity only. Script references carry the
        // previous execution generation and cannot resolve them in this run.
        self.active_children = 0;
    }

    pub(super) fn push(&mut self, register_count: usize, parent: Option<FrameId>) -> FrameId {
        let index = self.active_children;
        self.active_children += 1;
        if let Some(frame) = self.children.get_mut(index) {
            frame.reset_with_count(register_count, parent);
        } else {
            self.children.push(Frame::new(register_count, parent));
        }
        FrameId(self.active_children)
    }

    pub(super) fn get(&self, frame: FrameId) -> &Frame {
        if frame.0 == 0 {
            &self.root
        } else {
            debug_assert!(frame.0 <= self.active_children);
            &self.children[frame.0 - 1]
        }
    }

    pub(super) fn get_mut(&mut self, frame: FrameId) -> &mut Frame {
        if frame.0 == 0 {
            &mut self.root
        } else {
            debug_assert!(frame.0 <= self.active_children);
            &mut self.children[frame.0 - 1]
        }
    }

    pub(super) fn reset(&mut self, frame: FrameId, parent: Option<FrameId>) {
        let frame = self.get_mut(frame);
        frame.reset(parent);
    }
}

#[cfg(test)]
mod tests {
    use crate::{MiraValue, interpreter::RegisterId};

    use super::*;

    #[test]
    fn reuses_frames_without_leaking_register_values_between_runs() {
        let mut arena = FrameArena::new(20);
        arena
            .root
            .registers
            .get_mut(RegisterId::new(1))
            .replace(MiraValue::number(1.0));
        let child = arena.push(20, Some(FrameId::ROOT));
        arena
            .get_mut(child)
            .registers
            .get_mut(RegisterId::new(20))
            .replace(MiraValue::number(2.0));

        arena.begin_run(2);
        assert!(
            arena
                .root
                .registers
                .read(RegisterId::new(1))
                .is_uninitialized()
        );

        let reused = arena.push(2, Some(FrameId::ROOT));
        assert_eq!(reused, child);
        assert!(
            arena
                .get_mut(reused)
                .registers
                .read(RegisterId::new(1))
                .is_uninitialized()
        );
    }
}
