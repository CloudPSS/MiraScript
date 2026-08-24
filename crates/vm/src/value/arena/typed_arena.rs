use crate::{MiraError, Result, RuntimeErrorKind};

use super::{ArenaId, ArenaKey};

#[derive(Debug)]
pub(super) struct Arena<const CATEGORY: usize, T> {
    values: Vec<Option<T>>,
}

pub(super) const CAT_STRING: usize = 1;
pub(super) const CAT_RECORD: usize = 2;
pub(super) const CAT_ARRAY: usize = 3;
pub(super) const CAT_FUNCTION: usize = 4;
pub(super) const CAT_MODULE: usize = 5;

const fn category_name(category: usize) -> &'static str {
    match category {
        CAT_STRING => "string",
        CAT_RECORD => "record",
        CAT_ARRAY => "array",
        CAT_FUNCTION => "function",
        CAT_MODULE => "module",
        _ => panic!("invalid arena category"),
    }
}

impl<const CATEGORY: usize, T> Arena<CATEGORY, T> {
    pub fn new() -> Self {
        debug_assert!(!category_name(CATEGORY).is_empty());
        Self { values: Vec::new() }
    }

    fn check_key(&self, arena_id: ArenaId, key: ArenaKey) -> Result<()> {
        if key.arena_id() != arena_id {
            return Err(MiraError::runtime(RuntimeErrorKind::ForeignHandle));
        }
        if key.index() >= self.values.len() {
            return Err(MiraError::runtime(RuntimeErrorKind::InvalidHandle {
                category: category_name(CATEGORY),
            }));
        }
        let value = &self.values[key.index()];
        if value.is_none() {
            return Err(MiraError::runtime(RuntimeErrorKind::InvalidHandle {
                category: category_name(CATEGORY),
            }));
        }
        Ok(())
    }

    pub fn insert(&mut self, arena_id: ArenaId, value: T) -> Result<ArenaKey> {
        let key = ArenaKey::new(arena_id, self.values.len(), category_name(CATEGORY))?;
        self.values.push(Some(value));
        Ok(key)
    }

    pub fn get(&self, arena_id: ArenaId, key: ArenaKey) -> Result<&T> {
        self.check_key(arena_id, key)?;
        // SAFETY: The key has been validated to be within bounds and the value is guaranteed to be Some.
        unsafe { Ok(self.values.get_unchecked(key.index()).as_ref().unwrap()) }
    }

    pub fn get_mut(&mut self, arena_id: ArenaId, key: ArenaKey) -> Result<&mut T> {
        self.check_key(arena_id, key)?;
        // SAFETY: The key has been validated to be within bounds and the value is guaranteed to be Some.
        unsafe { Ok(self.values.get_unchecked_mut(key.index()).as_mut().unwrap()) }
    }

    pub fn take(&mut self, arena_id: ArenaId, key: ArenaKey) -> Result<T> {
        self.check_key(arena_id, key)?;
        // SAFETY: The key has been validated to be within bounds and the value is guaranteed to be Some.
        unsafe {
            Ok(self
                .values
                .get_unchecked_mut(key.index())
                .take()
                .unwrap_unchecked())
        }
    }
}
