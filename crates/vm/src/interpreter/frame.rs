use super::register::Registers;

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
        self.registers.fill(None);
        self.parent = parent;
    }
}

pub(super) struct FrameArena {
    root: Frame,
    children: Vec<Frame>,
}

impl FrameArena {
    pub(super) fn new(root_register_count: usize) -> Self {
        Self {
            root: Frame::new(root_register_count, None),
            children: Vec::new(),
        }
    }

    pub(super) fn push(&mut self, frame: Frame) -> FrameId {
        self.children.push(frame);
        FrameId(self.children.len())
    }

    pub(super) fn get(&self, frame: FrameId) -> &Frame {
        if frame.0 == 0 {
            &self.root
        } else {
            &self.children[frame.0 - 1]
        }
    }

    pub(super) fn get_mut(&mut self, frame: FrameId) -> &mut Frame {
        if frame.0 == 0 {
            &mut self.root
        } else {
            &mut self.children[frame.0 - 1]
        }
    }

    pub(super) fn reset(&mut self, frame: FrameId, parent: Option<FrameId>) {
        let frame = self.get_mut(frame);
        frame.reset(parent);
    }
}
