use crate::{MiraFunction, MiraHandle};

const INLINE_CALL_DEPTH: usize = 4;

pub(super) struct CallStack {
    inline: [Option<MiraHandle<dyn MiraFunction>>; INLINE_CALL_DEPTH],
    overflow: Vec<MiraHandle<dyn MiraFunction>>,
    len: usize,
}

impl CallStack {
    pub(super) fn new() -> Self {
        Self {
            inline: [None; INLINE_CALL_DEPTH],
            overflow: Vec::new(),
            len: 0,
        }
    }

    pub(super) fn depth(&self) -> usize {
        self.len
    }

    pub(super) fn push(&mut self, function: MiraHandle<dyn MiraFunction>) {
        if self.len < INLINE_CALL_DEPTH {
            self.inline[self.len] = Some(function);
        } else {
            self.overflow.push(function);
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

    pub(super) fn last(&self) -> Option<MiraHandle<dyn MiraFunction>> {
        if self.len == 0 {
            None
        } else if self.len <= INLINE_CALL_DEPTH {
            self.inline[self.len - 1]
        } else {
            self.overflow.last().copied()
        }
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = MiraHandle<dyn MiraFunction>> + '_ {
        self.inline[..self.len.min(INLINE_CALL_DEPTH)]
            .iter()
            .filter_map(|function| *function)
            .chain(self.overflow.iter().copied())
    }
}
