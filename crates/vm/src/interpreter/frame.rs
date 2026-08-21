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
    #[inline]
    pub fn new(register_count: usize, parent: Option<FrameId>) -> Self {
        Self {
            registers: Registers::new(register_count),
            parent,
        }
    }

    #[inline]
    pub fn reset(&mut self, parent: Option<FrameId>) {
        self.registers.fill(None);
        self.parent = parent;
    }
}
