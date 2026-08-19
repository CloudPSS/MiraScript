use paste::paste;
use std::{any::Any, ops::Deref};

use crate::interpreter::Runtime;

use super::types::{MiraArray, MiraFunction, MiraModule, MiraRecord};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ArenaKey(usize);

#[derive(Debug)]
struct Arena<T> {
    vec: Vec<Option<T>>,
}

impl<T> Arena<T> {
    fn new() -> Self {
        Self { vec: Vec::new() }
    }

    fn insert(&mut self, value: T) -> ArenaKey {
        let key = ArenaKey(self.vec.len());
        self.vec.push(Some(value));
        key
    }

    fn remove(&mut self, key: ArenaKey) -> Option<T> {
        self.vec.get_mut(key.0).and_then(|slot| slot.take())
    }

    fn get(&self, key: ArenaKey) -> Option<&T> {
        self.vec.get(key.0).and_then(|slot| slot.as_ref())
    }
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

pub(crate) struct MiraArena {
    strings: Arena<String>,
    arrays: Arena<Box<dyn MiraArray>>,
    records: Arena<Box<dyn MiraRecord>>,
    functions: Arena<Box<dyn MiraFunction>>,
    modules: Arena<Box<dyn MiraModule>>,
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

impl Runtime<'_> {
    pub fn [<insert_ $field>]<T: $ty>(&mut self, value: T) -> MiraHandle<T> {
        let key = self.arena.$field.insert(Box::new(value));
        MiraHandle {
            key,
            _marker: std::marker::PhantomData::<T>,
        }
    }

    pub fn [<take_ $field>]<T: $ty>(&mut self, handle: MiraHandle<T>) -> Option<T> {
        let value = self.arena.$field.remove(handle.key)?;
        let value: Box<dyn Any> = value;
        value.downcast::<T>().ok().map(|boxed| *boxed)
    }

    pub fn [<get_ $field>]<T: $ty>(&self, handle: MiraHandle<T>) -> Option<&T> {
        let value = self.arena.$field.get(handle.key)?;
        let value: &dyn Any = value.as_ref();
        value.downcast_ref::<T>()
    }
}
        })* };
}

impl MiraArena {
    pub fn new() -> Self {
        Self {
            strings: Arena::new(),
            arrays: Arena::new(),
            records: Arena::new(),
            functions: Arena::new(),
            modules: Arena::new(),
        }
    }
}

impl Runtime<'_> {
    pub fn insert_string(&mut self, value: impl Into<String>) -> MiraHandle<String> {
        let key = self.arena.strings.insert(value.into());
        MiraHandle {
            key,
            _marker: std::marker::PhantomData::<String>,
        }
    }

    pub fn take_string(&mut self, handle: MiraHandle<String>) -> Option<String> {
        self.arena.strings.remove(handle.key)
    }

    pub fn get_string(&self, handle: MiraHandle<String>) -> Option<&str> {
        self.arena.strings.get(handle.key).map(Deref::deref)
    }
}

impl_arena_handle! {
    MiraArray: arrays,
    MiraRecord: records,
    MiraFunction: functions,
    MiraModule: modules,
}
