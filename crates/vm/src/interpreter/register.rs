use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RegisterId(usize);

impl RegisterId {
    #[inline]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    #[inline]
    const fn is_nil(self) -> bool {
        self.0 == 0
    }
}

impl From<usize> for RegisterId {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value)
    }
}

const INLINE_REGISTER_COUNT: usize = 8;

#[derive(Debug)]
pub(super) struct Registers {
    inline: [Option<MiraValue>; INLINE_REGISTER_COUNT],
    overflow: Box<[Option<MiraValue>]>,
    #[cfg(debug_assertions)]
    pub(super) max_register: usize,
}

impl Registers {
    #[inline]
    pub(super) fn new(count: usize) -> Self {
        let need_overflow = count > INLINE_REGISTER_COUNT;
        if need_overflow {
            let overflow_count = count - INLINE_REGISTER_COUNT;
            Self {
                inline: [None; INLINE_REGISTER_COUNT],
                overflow: vec![None; overflow_count].into_boxed_slice(),
                #[cfg(debug_assertions)]
                max_register: count,
            }
        } else {
            Self {
                inline: [None; INLINE_REGISTER_COUNT],
                overflow: Box::new([]),
                #[cfg(debug_assertions)]
                max_register: count,
            }
        }
    }

    #[inline]
    pub(super) fn fill(&mut self, value: Option<MiraValue>) {
        for slot in self.inline.iter_mut() {
            *slot = value;
        }
        for slot in self.overflow.iter_mut() {
            *slot = value;
        }
    }

    fn check(&self, register: RegisterId) {
        let _id = register.0;
        #[cfg(debug_assertions)]
        debug_assert!(
            _id <= self.max_register,
            "register index {} out of bounds (max {})",
            _id,
            self.max_register
        );
    }

    #[inline]
    fn get(&self, register: RegisterId) -> &Option<MiraValue> {
        self.check(register);
        let id = register.0 - 1;
        if id < INLINE_REGISTER_COUNT {
            &self.inline[id]
        } else {
            &self.overflow[id - INLINE_REGISTER_COUNT]
        }
    }

    #[inline]
    pub(super) fn get_mut(&mut self, register: RegisterId) -> &mut Option<MiraValue> {
        self.check(register);
        let id = register.0 - 1;
        if id < INLINE_REGISTER_COUNT {
            &mut self.inline[id]
        } else {
            &mut self.overflow[id - INLINE_REGISTER_COUNT]
        }
    }
}

impl Runtime {
    #[inline]
    pub(super) fn read_register_raw(
        &self,
        frame: FrameId,
        register: impl Into<RegisterId>,
    ) -> Option<MiraValue> {
        let register = register.into();
        if register.is_nil() {
            Some(MiraValue::Nil)
        } else {
            *self.frames.get(frame).registers.get(register)
        }
    }

    #[inline]
    pub(super) fn read_register(
        &self,
        frame: FrameId,
        register: impl Into<RegisterId>,
    ) -> Result<MiraValue> {
        let register = register.into();
        self.read_register_raw(frame, register)
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::UninitializedValue))
    }

    #[inline]
    pub(super) fn read_number(
        &self,
        frame: FrameId,
        register: impl Into<RegisterId>,
    ) -> Result<f64> {
        let register = register.into();
        operations::to_number(self, self.read_register(frame, register)?)
    }

    #[inline]
    pub(super) fn write_register_raw(
        &mut self,
        frame: FrameId,
        register: impl Into<RegisterId>,
        value: Option<MiraValue>,
    ) {
        let register = register.into();
        if !register.is_nil() {
            *self.frames.get_mut(frame).registers.get_mut(register) = value;
        }
    }

    #[inline]
    pub(super) fn write_register(
        &mut self,
        frame: FrameId,
        register: impl Into<RegisterId>,
        value: MiraValue,
    ) {
        self.write_register_raw(frame, register, Some(value));
    }

    #[inline]
    pub(super) fn clear_register(&mut self, frame: FrameId, register: impl Into<RegisterId>) {
        self.write_register_raw(frame, register, None);
    }
}
