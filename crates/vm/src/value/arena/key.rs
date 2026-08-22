use crate::{MiraError, Result, RuntimeErrorKind};

use super::ArenaId;

const ARENA_INDEX_BITS: u32 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct ArenaKey(u64);

impl ArenaKey {
    pub fn new(arena_id: ArenaId, index: usize, category: &'static str) -> Result<Self> {
        let index = u32::try_from(index)
            .ok()
            .and_then(|index| index.checked_add(1))
            .ok_or_else(|| MiraError::runtime(RuntimeErrorKind::ArenaExhausted { category }))?;
        Ok(Self(
            (u64::from(arena_id.get()) << ARENA_INDEX_BITS) | u64::from(index),
        ))
    }

    pub fn arena_id(self) -> ArenaId {
        ArenaId::from((self.0 >> ARENA_INDEX_BITS) as u16)
    }

    pub fn index(self) -> usize {
        ((self.0 as u32) - 1) as usize
    }

    pub fn payload(self) -> [u8; 6] {
        let bytes = self.0.to_le_bytes();
        bytes[..6].try_into().expect("six-byte arena key")
    }

    pub fn from_payload(payload: [u8; 6]) -> Self {
        let mut bytes = [0; 8];
        bytes[..6].copy_from_slice(&payload);
        Self(u64::from_le_bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;

    use super::*;

    #[test]
    fn arena_key_roundtrips_through_six_byte_payload() {
        for (arena_id, index) in [
            (NonZeroU16::MIN, 0),
            (NonZeroU16::new(17).unwrap(), 42),
            (NonZeroU16::MAX, u32::MAX as usize - 1),
        ] {
            let arena_id = ArenaId::from(arena_id.get());
            let key = ArenaKey::new(arena_id, index, "test").unwrap();
            assert_eq!(ArenaKey::from_payload(key.payload()), key);
            assert_eq!(key.arena_id(), arena_id);
            assert_eq!(key.index(), index);
        }
    }

    #[test]
    fn arena_key_rejects_a_slot_outside_the_32_bit_component() {
        assert!(ArenaKey::new(ArenaId::next(), u32::MAX as usize, "test").is_err());
    }
}
