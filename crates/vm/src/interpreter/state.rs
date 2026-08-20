use super::*;

/// `None` is reserved for an uninitialized VM register.
pub(crate) type MiraAny = Option<MiraValue>;

/// Identifies a frame in the call stack.
/// Root frame is always `0`, and child frames are numbered sequentially starting from `1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct FrameId(usize);

pub(super) const ROOT_FRAME_ID: FrameId = FrameId(0);

pub(super) struct Frame {
    pub(super) registers: Vec<MiraAny>,
    pub(super) parent: Option<FrameId>,
}

pub(super) struct FrameArena {
    root: Frame,
    children: Vec<Frame>,
}

impl FrameArena {
    pub(super) fn new(root_register_count: usize) -> Self {
        Self {
            root: Frame {
                registers: vec![None; root_register_count + 1],
                parent: None,
            },
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
        frame.registers.fill(None);
        frame.parent = parent;
    }
}

pub(super) struct CallStack {
    inline: [Option<Rc<str>>; INLINE_CALL_DEPTH],
    overflow: Vec<Option<Rc<str>>>,
    len: usize,
}

impl CallStack {
    pub(super) fn new() -> Self {
        Self {
            inline: std::array::from_fn(|_| None),
            overflow: Vec::new(),
            len: 0,
        }
    }

    pub(super) fn push(&mut self, name: Option<Rc<str>>) {
        if self.len < INLINE_CALL_DEPTH {
            self.inline[self.len] = name;
        } else {
            self.overflow.push(name);
        }
        self.len += 1;
    }

    pub(super) fn pop(&mut self) {
        if self.len == 0 {
            return;
        }
        self.len -= 1;
        if self.len < INLINE_CALL_DEPTH {
            self.inline[self.len] = None;
        } else {
            self.overflow.pop();
        }
    }

    pub(super) fn last(&self) -> Option<&Option<Rc<str>>> {
        if self.len == 0 {
            None
        } else if self.len <= INLINE_CALL_DEPTH {
            Some(&self.inline[self.len - 1])
        } else {
            self.overflow.last()
        }
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &Option<Rc<str>>> {
        self.inline[..self.len.min(INLINE_CALL_DEPTH)]
            .iter()
            .chain(self.overflow.iter())
    }
}
