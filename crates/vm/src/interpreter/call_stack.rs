use crate::FunctionName;

const INLINE_CALL_DEPTH: usize = 4;

pub(super) struct CallStack {
    inline: [Option<FunctionName>; INLINE_CALL_DEPTH],
    overflow: Vec<FunctionName>,
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

    pub(super) fn depth(&self) -> usize {
        self.len
    }

    pub(super) fn push(&mut self, name: FunctionName) {
        if self.len < INLINE_CALL_DEPTH {
            self.inline[self.len] = Some(name);
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

    pub(super) fn last(&self) -> Option<&FunctionName> {
        if self.len == 0 {
            None
        } else if self.len <= INLINE_CALL_DEPTH {
            self.inline[self.len - 1].as_ref()
        } else {
            self.overflow.last()
        }
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &FunctionName> {
        self.inline[..self.len.min(INLINE_CALL_DEPTH)]
            .iter()
            .filter_map(|x| x.as_ref())
            .chain(self.overflow.iter())
    }
}
