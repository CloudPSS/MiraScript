use std::{
    num::NonZeroU16,
    sync::atomic::{AtomicU16, Ordering},
};

static NEXT_ARENA_ID: AtomicU16 = AtomicU16::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct ArenaId(NonZeroU16);

impl ArenaId {
    pub fn next() -> Self {
        Self::next_from(&NEXT_ARENA_ID)
    }

    fn next_from(next_id: &AtomicU16) -> Self {
        let id = next_id
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| {
                let id = NonZeroU16::new(id).expect("next arena identifier must be non-zero");
                Some(Self(id).wrapping_next().get())
            })
            .expect("arena identifier update cannot fail");
        Self(NonZeroU16::new(id).expect("arena identifier must be non-zero"))
    }

    fn wrapping_next(self) -> Self {
        Self(NonZeroU16::new(self.get().wrapping_add(1)).unwrap_or(NonZeroU16::MIN))
    }

    pub fn get(self) -> u16 {
        self.0.get()
    }

    pub fn from(id: u16) -> Self {
        Self(NonZeroU16::new(id).expect("arena key identifier must be non-zero"))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn arena_id_wraps_without_producing_zero() {
        let next_id = AtomicU16::new(u16::MAX);

        assert_eq!(ArenaId::next_from(&next_id), ArenaId(NonZeroU16::MAX));
        assert_eq!(ArenaId::next_from(&next_id), ArenaId(NonZeroU16::MIN));
        assert_eq!(next_id.load(Ordering::Relaxed), 2);
    }
}
