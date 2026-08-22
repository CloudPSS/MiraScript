use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MiraAny(MiraValue);

const _: () = assert!(std::mem::size_of::<MiraAny>() == 8);

impl MiraAny {
    #[inline]
    fn uninitialized() -> Self {
        Self(MiraValue::uninitialized())
    }

    #[inline]
    fn from_option(value: Option<MiraValue>) -> Self {
        match value {
            Some(value) => Self(value),
            None => Self::uninitialized(),
        }
    }

    #[inline]
    fn to_option(self) -> Option<MiraValue> {
        (!self.0.is_uninitialized()).then_some(self.0)
    }

    #[inline]
    pub(super) fn replace(&mut self, value: MiraValue) -> Option<MiraValue> {
        std::mem::replace(self, Self(value)).to_option()
    }
}

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
    inline: [MiraAny; INLINE_REGISTER_COUNT],
    overflow: Vec<MiraAny>,
    #[cfg(debug_assertions)]
    pub(super) max_register: usize,
}

impl Registers {
    #[inline]
    pub(super) fn new(count: usize) -> Self {
        Self {
            inline: std::array::from_fn(|_| MiraAny::uninitialized()),
            overflow: vec![MiraAny::uninitialized(); count.saturating_sub(INLINE_REGISTER_COUNT)],
            #[cfg(debug_assertions)]
            max_register: count,
        }
    }

    #[inline]
    pub(super) fn reset(&mut self, count: usize) {
        self.inline.fill(MiraAny::uninitialized());
        self.overflow.resize(
            count.saturating_sub(INLINE_REGISTER_COUNT),
            MiraAny::uninitialized(),
        );
        self.overflow.fill(MiraAny::uninitialized());
        #[cfg(debug_assertions)]
        {
            self.max_register = count;
        }
    }

    #[inline]
    pub(super) fn fill(&mut self, value: Option<MiraValue>) {
        for slot in self.inline.iter_mut() {
            *slot = MiraAny::from_option(value);
        }
        for slot in self.overflow.iter_mut() {
            *slot = MiraAny::from_option(value);
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
    pub(super) fn get(&self, register: RegisterId) -> &MiraAny {
        self.check(register);
        let id = register.0 - 1;
        if id < INLINE_REGISTER_COUNT {
            &self.inline[id]
        } else {
            &self.overflow[id - INLINE_REGISTER_COUNT]
        }
    }

    #[inline]
    pub(super) fn get_mut(&mut self, register: RegisterId) -> &mut MiraAny {
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
            Some(MiraValue::nil())
        } else {
            self.get(register).to_option()
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
            Some(MiraValue::nil())
        } else {
            self.frames.get(frame).registers.get(register).to_option()
        }
    }

    #[inline]
    pub(super) fn read_register(&self, frame: FrameId, register: RegisterId) -> Result<MiraValue> {
        self.read_register_raw(frame, register)
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::UninitializedValue))
    }

    #[inline]
    pub(super) fn read_number(&self, frame: FrameId, register: RegisterId) -> Result<f64> {
        let value = self.read_register_raw(frame, register);
        if let Some(number) = value.as_ref().and_then(MiraValue::as_number) {
            return Ok(number);
        }
        let Some(value) = value else {
            return Err(MiraError::runtime(RuntimeErrorKind::UninitializedValue));
        };
        operations::to_number(self, value)
    }

    #[inline]
    pub(super) fn write_register_raw(
        &mut self,
        frame: FrameId,
        register: RegisterId,
        value: Option<MiraValue>,
    ) {
        if !register.is_nil() {
            *self.frames.get_mut(frame).registers.get_mut(register) = MiraAny::from_option(value);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mira_any_is_copy_and_eight_bytes() {
        fn assert_copy<T: Copy>() {}

        assert_copy::<MiraAny>();
        assert_eq!(std::mem::size_of::<MiraAny>(), 8);
    }

    #[test]
    fn uninitialized_is_distinct_from_nil() {
        assert_eq!(MiraAny::uninitialized().to_option(), None);
        assert_eq!(
            MiraAny::from_option(Some(MiraValue::nil())).to_option(),
            Some(MiraValue::nil())
        );
    }
}
