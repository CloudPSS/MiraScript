use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct MiraAny(MiraValue);

const _: () = assert!(std::mem::size_of::<MiraAny>() == 8);

impl MiraAny {
    #[inline]
    pub fn uninitialized() -> Self {
        Self(MiraValue::UNINITIALIZED)
    }

    #[inline]
    pub fn replace(&mut self, value: MiraValue) -> MiraAny {
        std::mem::replace(self, Self(value))
    }

    #[inline]
    pub fn unwrap(self) -> MiraValue {
        self.0
    }
    #[inline]
    pub fn check(self) -> Result<MiraValue> {
        if self.is_uninitialized() {
            Err(MiraError::runtime(RuntimeErrorKind::UninitializedValue))
        } else {
            Ok(self.0)
        }
    }
    #[inline]
    pub fn is_uninitialized(self) -> bool {
        self.0.is_uninitialized()
    }
}

impl From<MiraValue> for MiraAny {
    #[inline]
    fn from(value: MiraValue) -> Self {
        Self(value)
    }
}
