use super::*;

pub(super) struct Frame {
    pub(super) registers: Vec<MiraAny>,
    pub(super) parent: Option<usize>,
}

pub(super) struct FrameArena {
    root: Frame,
    children: Vec<Frame>,
}

impl FrameArena {
    pub(super) fn new(root_register_count: usize) -> Self {
        Self {
            root: Frame {
                registers: vec![MiraAny::Uninitialized; root_register_count + 1],
                parent: None,
            },
            children: Vec::new(),
        }
    }

    pub(super) fn push(&mut self, frame: Frame) -> usize {
        self.children.push(frame);
        self.children.len()
    }

    pub(super) fn get(&self, frame: usize) -> &Frame {
        if frame == 0 {
            &self.root
        } else {
            &self.children[frame - 1]
        }
    }

    pub(super) fn get_mut(&mut self, frame: usize) -> &mut Frame {
        if frame == 0 {
            &mut self.root
        } else {
            &mut self.children[frame - 1]
        }
    }

    pub(super) fn reset(&mut self, frame: usize, parent: Option<usize>) {
        let frame = self.get_mut(frame);
        frame.registers.fill(MiraAny::Uninitialized);
        frame.parent = parent;
    }
}

pub(super) struct CallStack {
    inline: [Option<Rc<str>>; INLINE_CALL_DEPTH],
    overflow: Vec<Option<Rc<str>>>,
    len: usize,
}

pub(super) enum GlobalSlots<'a> {
    Empty,
    One(Option<&'a MiraAny>),
    Two([Option<&'a MiraAny>; 2]),
    Inline([Option<&'a MiraAny>; INLINE_GLOBAL_SLOTS]),
    Overflow(Vec<Option<&'a MiraAny>>),
}

impl<'a> GlobalSlots<'a> {
    pub(super) fn new(names: &[String], context: &'a MiraContext) -> Self {
        if names.is_empty() {
            Self::Empty
        } else if names.len() == 1 {
            Self::One(context.get_ref(&names[0]))
        } else if names.len() == 2 {
            Self::Two(std::array::from_fn(|index| context.get_ref(&names[index])))
        } else if names.len() <= INLINE_GLOBAL_SLOTS {
            Self::Inline(std::array::from_fn(|index| {
                names.get(index).and_then(|name| context.get_ref(name))
            }))
        } else {
            Self::Overflow(names.iter().map(|name| context.get_ref(name)).collect())
        }
    }

    pub(super) fn get_ref(&self, slot: usize) -> Option<&'a MiraAny> {
        match self {
            Self::Empty => None,
            Self::One(value) => *value,
            Self::Two(values) => values[slot],
            Self::Inline(values) => values[slot],
            Self::Overflow(values) => values[slot],
        }
    }
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
