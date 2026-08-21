use crate::MiraValue::Nil;

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RegisterId(usize);

impl RegisterId {
    #[inline]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    #[inline]
    pub const fn is_nil(self) -> bool {
        self.0 == 0
    }
}

const INLINE_REGISTER_COUNT: usize = 16;

#[derive(Debug)]
pub(super) struct Registers {
    inline: [Option<MiraValue>; INLINE_REGISTER_COUNT],
    overflow: Vec<Option<MiraValue>>,
    #[cfg(debug_assertions)]
    pub(super) max_register: usize,
}

impl Registers {
    #[inline]
    pub(super) fn new(count: usize) -> Self {
        Self {
            inline: [None; INLINE_REGISTER_COUNT],
            overflow: vec![None; count.saturating_sub(INLINE_REGISTER_COUNT)],
            #[cfg(debug_assertions)]
            max_register: count,
        }
    }

    #[inline]
    pub(super) fn reset(&mut self, count: usize) {
        self.inline.fill(None);
        self.overflow
            .resize(count.saturating_sub(INLINE_REGISTER_COUNT), None);
        self.overflow.fill(None);
        #[cfg(debug_assertions)]
        {
            self.max_register = count;
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

    #[inline(always)]
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
    pub(super) fn get(&self, register: RegisterId) -> &Option<MiraValue> {
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

    #[inline]
    pub(super) fn read(&self, register: RegisterId) -> Option<MiraValue> {
        if register.is_nil() {
            Some(Nil)
        } else {
            *self.get(register)
        }
    }
}

impl Runtime {
    #[inline]
    pub(super) fn read_register_raw(
        &self,
        frame: FrameId,
        register: RegisterId,
    ) -> Option<MiraValue> {
        if register.is_nil() {
            Some(MiraValue::Nil)
        } else {
            *self.frames.get(frame).registers.get(register)
        }
    }

    #[inline]
    pub(super) fn read_register(&self, frame: FrameId, register: RegisterId) -> Result<MiraValue> {
        self.read_register_raw(frame, register)
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::UninitializedValue))
    }

    #[inline]
    pub(super) fn read_number(&self, frame: FrameId, register: RegisterId) -> Result<f64> {
        operations::to_number(self, self.read_register(frame, register)?)
    }

    #[inline]
    pub(super) fn write_register_raw(
        &mut self,
        frame: FrameId,
        register: RegisterId,
        value: Option<MiraValue>,
    ) {
        if !register.is_nil() {
            *self.frames.get_mut(frame).registers.get_mut(register) = value;
        }
    }

    #[inline]
    pub(super) fn write_register(
        &mut self,
        frame: FrameId,
        register: RegisterId,
        value: MiraValue,
    ) {
        self.write_register_raw(frame, register, Some(value));
    }

    #[inline]
    pub(super) fn clear_register(&mut self, frame: FrameId, register: RegisterId) {
        self.write_register_raw(frame, register, None);
    }
}
