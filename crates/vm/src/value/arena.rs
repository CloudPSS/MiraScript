use paste::paste;
use std::{any::Any, ops::Deref};

use slotmap::{SlotMap, new_key_type};

use super::types::{MiraArray, MiraFunction, MiraModule, MiraRecord};

new_key_type! {
    struct ArenaKey;
}

pub struct MiraHandle<T: Any + ?Sized> {
    key: ArenaKey,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Any + ?Sized> Copy for MiraHandle<T> {}
impl<T: Any + ?Sized> Clone for MiraHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Any + ?Sized> std::fmt::Debug for MiraHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MiraHandle").field(&self.key.0).finish()
    }
}

impl<T: Any + ?Sized> PartialEq for MiraHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

pub struct MiraArena {
    strings: SlotMap<ArenaKey, String>,
    arrays: SlotMap<ArenaKey, Box<dyn MiraArray>>,
    records: SlotMap<ArenaKey, Box<dyn MiraRecord>>,
    functions: SlotMap<ArenaKey, Box<dyn MiraFunction>>,
    modules: SlotMap<ArenaKey, Box<dyn MiraModule>>,
}

macro_rules! impl_arena_handle {
    ($($ty:ty: $field:ident),* $(,)?) => {$ (paste! {
impl<T: $ty> From<MiraHandle<T>> for MiraHandle<dyn $ty> {
    fn from(handle: MiraHandle<T>) -> Self {
        MiraHandle {
            key: handle.key,
            _marker: std::marker::PhantomData::<dyn $ty>,
        }
    }
}

impl MiraArena {
    pub fn [<insert_ $field>]<T: $ty>(&mut self, value: T) -> MiraHandle<T> {
        let key = self.$field.insert(Box::new(value));
        MiraHandle {
            key,
            _marker: std::marker::PhantomData::<T>,
        }
    }

    pub fn [<take_ $field>]<T: $ty>(&mut self, handle: MiraHandle<T>) -> Option<T> {
        let value = self.$field.remove(handle.key)?;
        let value: Box<dyn Any> = value;
        value.downcast::<T>().ok().map(|boxed| *boxed)
    }

    pub fn [<get_ $field>]<T: $ty>(&self, handle: MiraHandle<T>) -> Option<&T> {
        let value = self.$field.get(handle.key)?;
        let value: &dyn Any = value.as_ref();
        value.downcast_ref::<T>()
    }
}
        })* };
}

impl MiraArena {
    pub fn new() -> Self {
        Self {
            strings: SlotMap::with_key(),
            arrays: SlotMap::with_key(),
            records: SlotMap::with_key(),
            functions: SlotMap::with_key(),
            modules: SlotMap::with_key(),
        }
    }

    pub fn insert_string(&mut self, value: impl Into<String>) -> MiraHandle<String> {
        let key = self.strings.insert(value.into());
        MiraHandle {
            key,
            _marker: std::marker::PhantomData::<String>,
        }
    }

    pub fn take_string(&mut self, handle: MiraHandle<String>) -> Option<String> {
        self.strings.remove(handle.key)
    }

    pub fn get_string(&self, handle: MiraHandle<String>) -> Option<&str> {
        self.strings.get(handle.key).map(Deref::deref)
    }
}

impl_arena_handle! {
    MiraArray: arrays,
    MiraRecord: records,
    MiraFunction: functions,
    MiraModule: modules,
}
